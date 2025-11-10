module bridge::bridge {
    use std::features;
    use std::signer;
    use std::string;
    use std::vector;
    use std::option;

    use cedra_std::table::{Self as table, Table};

    use cedra_framework::account;
    use cedra_framework::event::{Self as ev, EventHandle};
    use cedra_framework::system_addresses;

    // FA & Objects
    use cedra_framework::fungible_asset::{Self as fa, Metadata, MintRef, BurnRef, TransferRef, FungibleAsset};
    use cedra_framework::object::{Self as object, Object};
    use cedra_framework::primary_fungible_store;

    /* ===================== Admin (multisig) ===================== */
    struct Admin has key { multisig: address }

    /* ===================== Errors ===================== */
    const E_NOT_ADMIN: u64              = 1;
    const E_BAD_INPUT: u64              = 2;
    const E_ALREADY_ACTIVE: u64         = 5;
    const E_NONCE_USED: u64             = 9;
    const E_PAUSED: u64                 = 11;
    const E_ZERO_AMOUNT: u64            = 12;
    const E_ALREADY_INITIALIZED: u64    = 13;
    const E_NO_APPROVAL: u64            = 14;
    const E_APPROVAL_MISMATCH: u64      = 15;
    const E_ASSET_EXISTS: u64           = 16;
    const E_ASSET_UNKNOWN: u64          = 17;
    const E_ASSET_MISMATCH: u64         = 18; // registry mismatch vs approval

    /* ===================== Global config & registry (FA) ===================== */

    struct Config has key { paused: bool }

    // Per-asset refs, keyed by metadata object address
    struct FACaps has store {
        mint: MintRef,
        burn: BurnRef,
        transfer: TransferRef,
    }

    struct FARegistry has key {
        // L1 token (20 bytes) -> FA Metadata object
        l1_to_metadata: Table<vector<u8>, Object<Metadata>>,
        // Metadata object address -> FACaps
        caps_by_meta: Table<address, FACaps>,
    }

    /* ===================== Requests / Approvals ===================== */
    struct WithdrawalApproval has copy, drop, store {
        user: address,
        l1_token: vector<u8>,       // 20 bytes (ETH = all zeros in your convention)
        metadata_addr: address,     // bind approval to concrete FA object even if registry changes
        eth_recipient: vector<u8>,  // 20 bytes
        amount: u64,
    }

    struct Requests has key {
        used_nonce: Table<u64, bool>,
        approvals: Table<u64, WithdrawalApproval>,
    }

    /* ===================== Events ===================== */
    #[event]
    struct DepositObserved has drop, store {
        l1_token: vector<u8>,      // 20 bytes
        eth_tx_hash: vector<u8>,
        to: address,
        amount: u64,
        nonce: u64,
    }

    #[event]
    struct MintExecuted has drop, store {
        l1_token: vector<u8>,
        to: address,
        amount: u64,
        nonce: u64,
    }

    #[event]
    struct Withdrawal has drop, store {
        l1_token: vector<u8>,
        from: address,
        eth_recipient: vector<u8>, // 20 bytes
        amount: u64,
        nonce: u64,
    }

    struct BridgeEvents has key {
        deposit_observed: EventHandle<DepositObserved>,
        mint_executed:    EventHandle<MintExecuted>,
        withdrawal:       EventHandle<Withdrawal>,
    }

    /* ===================== Helpers ===================== */
    inline fun assert_not_paused() acquires Config {
        assert!(!borrow_global<Config>(@bridge).paused, E_PAUSED);
    }
    inline fun assert_framework(s: &signer) { 
        // DEV: allow the module owner (bridge address) to act as "framework"
        assert!(signer::address_of(s) == @bridge, E_NOT_ADMIN);
    }
    inline fun assert_multisig(s: &signer) acquires Admin {
        let admin = borrow_global<Admin>(@bridge);
        assert!(signer::address_of(s) == admin.multisig, E_NOT_ADMIN);
    }
    inline fun assert_20_bytes(addr: &vector<u8>) {
        assert!(vector::length(addr) == 20, E_BAD_INPUT);
    }

    inline fun get_registry(): &mut FARegistry {
        borrow_global_mut<FARegistry>(@bridge)
    }

    inline fun get_metadata_or_abort(l1: &vector<u8>): Object<Metadata> acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
        assert!(table::contains(&reg.l1_to_metadata, *l1), E_ASSET_UNKNOWN);
        *table::borrow(&reg.l1_to_metadata, *l1)
    }

    inline fun get_caps_or_abort(meta_addr: address): &mut FACaps acquires FARegistry {
        let reg = borrow_global_mut<FARegistry>(@bridge);
        assert!(table::contains(&reg.caps_by_meta, meta_addr), E_ASSET_UNKNOWN);
        table::borrow_mut(&mut reg.caps_by_meta, meta_addr)
    }

    /* ===================== Init & admin rotation ===================== */

    public entry fun initialize(bridge_owner: &signer) {
        assert_framework(bridge_owner);
        assert!(!exists<Config>(@bridge), E_ALREADY_INITIALIZED);

        move_to(bridge_owner, Config { paused: false });

        move_to(bridge_owner, FARegistry {
            l1_to_metadata: table::new<vector<u8>, Object<Metadata>>(),
            caps_by_meta:   table::new<address, FACaps>(),
        });

        move_to(bridge_owner, Requests {
            used_nonce: table::new<u64, bool>(),
            approvals:  table::new<u64, WithdrawalApproval>(),
        });

        move_to(bridge_owner, BridgeEvents {
            deposit_observed: account::new_event_handle<DepositObserved>(bridge_owner),
            mint_executed:    account::new_event_handle<MintExecuted>(bridge_owner),
            withdrawal:       account::new_event_handle<Withdrawal>(bridge_owner),
        });

        move_to(bridge_owner, Admin { multisig: signer::address_of(bridge_owner) });
    }

    public entry fun set_multisig_framework_only(bridge_owner: &signer, addr: address) acquires Admin {
        assert_framework(bridge_owner);
        borrow_global_mut<Admin>(@bridge).multisig = addr;
    }

    public entry fun rotate_multisig(multisig: &signer, new_addr: address) acquires Admin {
        assert_multisig(multisig);
        borrow_global_mut<Admin>(@bridge).multisig = new_addr;
    }

    public fun pause(multisig: &signer) acquires Config, Admin {
        assert_multisig(multisig);
        borrow_global_mut<Config>(@bridge).paused = true;
    }
    public fun unpause(multisig: &signer) acquires Config, Admin {
        assert_multisig(multisig);
        borrow_global_mut<Config>(@bridge).paused = false;
    }

    /* ===================== Governance: manage asset registry (FA) ===================== */

    /// Create a new FA for an L1 token and store metadata + refs.
    /// All string-ish arguments are UTF-8 bytes (converted internally).
    public entry fun add_asset(
        bridge_owner: &signer,
        l1_token: vector<u8>,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
        icon_uri: vector<u8>,
        project_uri: vector<u8>,
    ) acquires FARegistry {
        assert_framework(bridge_owner);
        assert_20_bytes(&l1_token);

        let reg = get_registry();
        assert!(!table::contains(&reg.l1_to_metadata, l1_token), E_ASSET_EXISTS);

        // Create a non-deletable object; we can use the symbol (or l1_token) as object name.
        let ctor = &object::create_named_object(bridge_owner, symbol);

        // Create FA metadata and enable primary store auto-creation
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            ctor,
            option::none<u128>(),
            string::utf8(name),
            string::utf8(symbol),
            decimals,
            string::utf8(icon_uri),
            string::utf8(project_uri),
        );

        // Reconstruct the Metadata object for this FA
        let meta: Object<Metadata> = object::object_from_constructor_ref<Metadata>(ctor);

        // Generate refs at creation time
        let mint_ref     = fa::generate_mint_ref(ctor);
        let transfer_ref = fa::generate_transfer_ref(ctor);
        let burn_ref     = fa::generate_burn_ref(ctor);

        // Record in registry
        table::add(&mut reg.l1_to_metadata, l1_token, meta);
        table::add(
            &mut reg.caps_by_meta,
            object::object_address(&meta),
            FACaps { mint: mint_ref, burn: burn_ref, transfer: transfer_ref }
        );
    }

    /// Remove the L1->FA mapping (does not destroy metadata or refs).
    public entry fun remove_asset(bridge_owner: &signer, l1_token: vector<u8>) acquires FARegistry {
        assert_framework(bridge_owner);
        assert_20_bytes(&l1_token);

        let reg = get_registry();
        assert!(table::contains(&reg.l1_to_metadata, l1_token), E_ASSET_UNKNOWN);
        let _ = table::remove(&mut reg.l1_to_metadata, l1_token);
        // We deliberately keep caps entry so remaining holders can still operate/burn if you want that.
        // If you want to freeze post-delist, you can use transfer_ref to freeze accounts externally.
    }

    /* ===================== Deposit / Mint (multisig-gated) ===================== */

    public entry fun execute_deposit(
        multisig: &signer,
        l1_token: vector<u8>,
        to: address,
        amount: u64,
        nonce: u64,
        eth_tx_hash: vector<u8>,
    ) acquires FARegistry, Requests, BridgeEvents, Config, Admin {
        assert_not_paused();
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&l1_token);

        let reqs = borrow_global_mut<Requests>(@bridge);
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);

        let meta = get_metadata_or_abort(&l1_token);
        let caps = get_caps_or_abort(object::object_address(&meta));

        let minted: FungibleAsset = fa::mint(&caps.mint, amount);
        primary_fungible_store::deposit(to, minted);

        table::add(&mut reqs.used_nonce, nonce, true);

        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(DepositObserved { l1_token, eth_tx_hash, to, amount, nonce });
            ev::emit(MintExecuted    { l1_token, to, amount, nonce });
        } else {
            ev::emit_event(&mut evs.deposit_observed, DepositObserved { l1_token: l1_token, eth_tx_hash, to, amount, nonce });
            ev::emit_event(&mut evs.mint_executed,   MintExecuted    { l1_token: l1_token, to, amount, nonce });
        };
    }

    /* ===================== Withdrawal (approval + user burn) ===================== */

    public entry fun approve_withdrawal(
        multisig: &signer,
        user: address,
        l1_token: vector<u8>,
        eth_recipient: vector<u8>,
        amount: u64,
        nonce: u64
    ) acquires FARegistry, Admin, Requests, Config {
        assert_not_paused();
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&l1_token);
        assert!(vector::length(&eth_recipient) == 20, E_BAD_INPUT);

        let reqs = borrow_global_mut<Requests>(@bridge);
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);
        assert!(!table::contains(&reqs.approvals, nonce), E_ALREADY_ACTIVE);

        let meta = get_metadata_or_abort(&l1_token);
        let metadata_addr = object::object_address(&meta);

        table::add(&mut reqs.approvals, nonce, WithdrawalApproval {
            user,
            l1_token,
            metadata_addr,
            eth_recipient,
            amount,
        });
    }

    public entry fun withdraw_to_l1(
        user: &signer,
        l1_token: vector<u8>,
        eth_recipient: vector<u8>,
        amount: u64,
        nonce: u64
    ) acquires FARegistry, BridgeEvents, Requests, Config {
        assert_not_paused();
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert!(vector::length(&eth_recipient) == 20, E_BAD_INPUT);
        assert_20_bytes(&l1_token);

        let reqs = borrow_global_mut<Requests>(@bridge);
        assert!(table::contains(&reqs.approvals, nonce), E_NO_APPROVAL);
        let appr = table::borrow(&reqs.approvals, nonce);

        // Full match
        assert!(appr.user == signer::address_of(user), E_APPROVAL_MISMATCH);
        assert!(appr.amount == amount, E_APPROVAL_MISMATCH);
        assert!(appr.eth_recipient == eth_recipient, E_APPROVAL_MISMATCH);
        assert!(appr.l1_token == l1_token, E_APPROVAL_MISMATCH);

        // Re-fetch metadata from registry and ensure it's the same object as approval
        let meta = get_metadata_or_abort(&l1_token);
        let meta_addr_now = object::object_address(&meta);
        assert!(meta_addr_now == appr.metadata_addr, E_ASSET_MISMATCH);

        // Withdraw from user's primary store, then burn with stored BurnRef
        let fa_withdrawn: FungibleAsset = primary_fungible_store::withdraw<Metadata>(user, meta, amount);
        let caps = get_caps_or_abort(meta_addr_now);
        fa::burn(&caps.burn, fa_withdrawn);

        // Mark nonce used + clear approval
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);
        table::add(&mut reqs.used_nonce, nonce, true);
        let _ = table::remove(&mut reqs.approvals, nonce);

        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(Withdrawal { l1_token, from: signer::address_of(user), eth_recipient, amount, nonce });
        } else {
            ev::emit_event(&mut evs.withdrawal, Withdrawal { l1_token, from: signer::address_of(user), eth_recipient, amount, nonce });
        };
    }

    /* ===================== Views ===================== */
    #[view]
    public fun admin_multisig(): address acquires Admin {
        borrow_global<Admin>(@bridge).multisig
    }

    #[view]
    public fun nonce_used(n: u64): bool acquires Requests {
        table::contains(&borrow_global<Requests>(@bridge).used_nonce, n)
    }

    #[view]
    public fun balance_of(l1: vector<u8>, owner: address): u64 acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
        // Use the table via `reg`, never by value
        assert!(table::contains(&reg.l1_to_metadata, l1), E_ASSET_UNKNOWN);
        let meta_obj: Object<Metadata> = *table::borrow(&reg.l1_to_metadata, l1);
        primary_fungible_store::balance<Metadata>(owner, meta_obj)
    }

    /* ===================== (Optional) UX helper ===================== */
    /// No-op with FA: primary store is auto-created on demand. Left for CLI parity.
    public entry fun ensure_store(_user: &signer) {}
}