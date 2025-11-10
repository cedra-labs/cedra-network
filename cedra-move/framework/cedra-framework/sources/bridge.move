module cedra_framework::bridge {
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
        assert!(!borrow_global<Config>(@cedra_framework).paused, E_PAUSED);
    }
    inline fun assert_framework(s: &signer) { system_addresses::assert_cedra_framework(s); }
    inline fun assert_multisig(s: &signer) acquires Admin {
        let admin = borrow_global<Admin>(@cedra_framework);
        assert!(signer::address_of(s) == admin.multisig, E_NOT_ADMIN);
    }
    inline fun assert_20_bytes(addr: &vector<u8>) {
        assert!(vector::length(addr) == 20, E_BAD_INPUT);
    }

    inline fun get_registry(): &mut FARegistry {
        borrow_global_mut<FARegistry>(@cedra_framework)
    }

    inline fun get_metadata_or_abort(l1: &vector<u8>): Object<Metadata> acquires FARegistry {
        let reg = borrow_global<FARegistry>(@cedra_framework);
        assert!(table::contains(&reg.l1_to_metadata, *l1), E_ASSET_UNKNOWN);
        *table::borrow(&reg.l1_to_metadata, *l1)
    }

    inline fun get_caps_or_abort(meta_addr: address): &mut FACaps acquires FARegistry {
        let reg = borrow_global_mut<FARegistry>(@cedra_framework);
        assert!(table::contains(&reg.caps_by_meta, meta_addr), E_ASSET_UNKNOWN);
        table::borrow_mut(&mut reg.caps_by_meta, meta_addr)
    }

    /* ===================== Init & admin rotation ===================== */

    public fun initialize(cedra_framework: &signer) {
        assert_framework(cedra_framework);
        assert!(!exists<Config>(@cedra_framework), E_ALREADY_INITIALIZED);

        move_to(cedra_framework, Config { paused: false });

        move_to(cedra_framework, FARegistry {
            l1_to_metadata: table::new<vector<u8>, Object<Metadata>>(),
            caps_by_meta:   table::new<address, FACaps>(),
        });

        move_to(cedra_framework, Requests {
            used_nonce: table::new<u64, bool>(),
            approvals:  table::new<u64, WithdrawalApproval>(),
        });

        move_to(cedra_framework, BridgeEvents {
            deposit_observed: account::new_event_handle<DepositObserved>(cedra_framework),
            mint_executed:    account::new_event_handle<MintExecuted>(cedra_framework),
            withdrawal:       account::new_event_handle<Withdrawal>(cedra_framework),
        });

        move_to(cedra_framework, Admin { multisig: signer::address_of(cedra_framework) });
    }

    public entry fun set_multisig_framework_only(cedra_framework: &signer, addr: address) acquires Admin {
        assert_framework(cedra_framework);
        borrow_global_mut<Admin>(@cedra_framework).multisig = addr;
    }

    public entry fun rotate_multisig(multisig: &signer, new_addr: address) acquires Admin {
        assert_multisig(multisig);
        borrow_global_mut<Admin>(@cedra_framework).multisig = new_addr;
    }

    public fun pause(multisig: &signer) acquires Config, Admin {
        assert_multisig(multisig);
        borrow_global_mut<Config>(@cedra_framework).paused = true;
    }
    public fun unpause(multisig: &signer) acquires Config, Admin {
        assert_multisig(multisig);
        borrow_global_mut<Config>(@cedra_framework).paused = false;
    }

    /* ===================== Governance: manage asset registry (FA) ===================== */

    /// Create a new FA for an L1 token and store metadata + refs.
    /// All string-ish arguments are UTF-8 bytes (converted internally).
    public entry fun add_asset(
        cedra_framework: &signer,
        l1_token: vector<u8>,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
        icon_uri: vector<u8>,
        project_uri: vector<u8>,
    ) acquires FARegistry {
        assert_framework(cedra_framework);
        assert_20_bytes(&l1_token);

        let reg = get_registry();
        assert!(!table::contains(&reg.l1_to_metadata, l1_token), E_ASSET_EXISTS);

        // Create a non-deletable object; we can use the symbol (or l1_token) as object name.
        let ctor = &object::create_named_object(cedra_framework, symbol);

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
    public entry fun remove_asset(cedra_framework: &signer, l1_token: vector<u8>) acquires FARegistry {
        assert_framework(cedra_framework);
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

        let reqs = borrow_global_mut<Requests>(@cedra_framework);
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);

        let meta = get_metadata_or_abort(&l1_token);
        let caps = get_caps_or_abort(object::object_address(&meta));

        let minted: FungibleAsset = fa::mint(&caps.mint, amount);
        primary_fungible_store::deposit(to, minted);

        table::add(&mut reqs.used_nonce, nonce, true);

        let evs = borrow_global_mut<BridgeEvents>(@cedra_framework);
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

        let reqs = borrow_global_mut<Requests>(@cedra_framework);
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

        let reqs = borrow_global_mut<Requests>(@cedra_framework);
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

        let evs = borrow_global_mut<BridgeEvents>(@cedra_framework);
        if (features::module_event_migration_enabled()) {
            ev::emit(Withdrawal { l1_token, from: signer::address_of(user), eth_recipient, amount, nonce });
        } else {
            ev::emit_event(&mut evs.withdrawal, Withdrawal { l1_token, from: signer::address_of(user), eth_recipient, amount, nonce });
        };
    }

    /* ===================== Views ===================== */
    #[view]
    public fun admin_multisig(): address acquires Admin {
        borrow_global<Admin>(@cedra_framework).multisig
    }

    #[view]
    public fun nonce_used(n: u64): bool acquires Requests {
        table::contains(&borrow_global<Requests>(@cedra_framework).used_nonce, n)
    }

    #[view]
    public fun balance_of(l1: vector<u8>, owner: address): u64 acquires FARegistry {
        let reg = borrow_global<FARegistry>(@cedra_framework);
        // Use the table via `reg`, never by value
        assert!(table::contains(&reg.l1_to_metadata, l1), E_ASSET_UNKNOWN);
        let meta_obj: Object<Metadata> = *table::borrow(&reg.l1_to_metadata, l1);
        primary_fungible_store::balance<Metadata>(owner, meta_obj)
    }

    /* ===================== (Optional) UX helper ===================== */
    /// No-op with FA: primary store is auto-created on demand. Left for CLI parity.
    public entry fun ensure_store(_user: &signer) {}

    /************** TESTS **************/
    #[test_only]
    use cedra_framework::bridge;

    // Mini-genesis for tests: create & init @cedra_framework, then init bridge.
    #[test_only]
    fun setup_framework_for_tests(): signer {
        let (cf, _cap) = account::create_framework_reserved_account(@cedra_framework);
        account::initialize(&cf);
        bridge::initialize(&cf);
        cf
    }

    // 20 ASCII '0' as placeholder for ETH L1 token key (length must be 20)
    const ETH_L1: vector<u8> = b"00000000000000000000";

    // Helper: create FA for ETH_L1 under governance signer &cf
    #[test_only]
    fun add_eth_asset(cf: &signer) acquires FARegistry{
        // name, symbol, decimals, empty icon/project URIs for tests
        bridge::add_asset(
            cf,
            ETH_L1,
            b"Wrapped Ether",
            b"WETH",
            18,
            b"",
            b""
        );
    }

    /// Mint executes when (and only when) the configured multisig calls `execute_deposit`.
    #[test(msig=@0x501, user=@0x2222)]
    public entry fun test_multisig_quorum_mints(msig: &signer, user: &signer) acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();
        // make msig the admin for runtime actions
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        // governance creates the FA for ETH_L1
        add_eth_asset(&cf);

        let u = signer::address_of(user);
        let before = bridge::balance_of(ETH_L1, u);

        // mint via multisig
        let amt = 250;
        let nonce = 42;
        bridge::execute_deposit(msig, ETH_L1, u, amt, nonce, b"eth-tx-42");

        let after = bridge::balance_of(ETH_L1, u);
        assert!(after == before + amt, 0x1001);
        assert!(bridge::nonce_used(nonce), 0x1002);
    }

    /// Rotating the admin multisig changes who can call the gated functions.
    #[test(msig1=@0xAAA, msig2=@0xBBB, to=@0x3333)]
    public entry fun test_admin_multisig_rotation(msig1: &signer, msig2: &signer, to: &signer) acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig1));
        add_eth_asset(&cf);

        let to_addr = signer::address_of(to);
        let b0 = bridge::balance_of(ETH_L1, to_addr);

        // Works for msig1
        bridge::execute_deposit(msig1, ETH_L1, to_addr, 100, 1, b"tx-1");

        // Rotate to msig2
        bridge::rotate_multisig(msig1, signer::address_of(msig2));
        assert!(bridge::admin_multisig() == signer::address_of(msig2), 0x1101);

        // Now only msig2 should work
        bridge::execute_deposit(msig2, ETH_L1, to_addr, 200, 2, b"tx-2");

        let b1 = bridge::balance_of(ETH_L1, to_addr);
        assert!(b1 == b0 + 100 + 200, 0x1102);
    }

    /// Paused blocks execute_deposit.
    #[test(msig=@0x501, to=@0x4444)]
    #[expected_failure(abort_code = 11)] // E_PAUSED
    public entry fun test_pause_blocks_paths_execute_deposit(msig: &signer, to: &signer) acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        add_eth_asset(&cf);

        bridge::pause(msig);
        bridge::execute_deposit(msig, ETH_L1, signer::address_of(to), 10, 9, b"tx-paused");
    }

    /// Full withdrawal happy path: multisig approves and user withdraws; nonce is consumed; approval is removed.
    #[test(msig=@0x501, user=@0x201)]
    public entry fun test_withdrawal_approval_and_execution(msig: &signer, user: &signer) acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();

        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        add_eth_asset(&cf);

        let u = signer::address_of(user);

        // fund user
        bridge::execute_deposit(msig, ETH_L1, u, 1000, 7001, b"tx-fund");

        let before = bridge::balance_of(ETH_L1, u);

        // Approve specific withdrawal: (user, l1_token, eth_recipient[20], amount, nonce)
        let recp = b"11111111111111111111"; // 20 bytes
        let amount = 150;
        let nonce = 9001;

        bridge::approve_withdrawal(msig, u, ETH_L1, recp, amount, nonce);

        // User executes (burn on L2, event for L1 payout)
        bridge::withdraw_to_l1(user, ETH_L1, recp, amount, nonce);

        let after = bridge::balance_of(ETH_L1, u);
        assert!(before == after + amount, 0x1201);
        assert!(bridge::nonce_used(nonce), 0x1202);
    }

        // Extra L1 keys for negative tests (length must be 20)
    const BAD_L1: vector<u8> = b"99999999999999999999";

    /// initialize() must not be callable twice.
    #[test]
    #[expected_failure(abort_code = 13)] // E_ALREADY_INITIALIZED
    public entry fun should_forbid_to_initialize_twice() {
        let cf = setup_framework_for_tests();
        // initialize() was already called inside setup_framework_for_tests()
        // Calling again should fail with E_ALREADY_INITIALIZED.
        bridge::initialize(&cf);
    }

    /// initialize() must only be callable by @cedra_framework.
    /// We don't assert an abort code here because it's thrown by system_addresses.
    #[test(attacker=@0x1111)]
    #[expected_failure] // comes from system_addresses::assert_cedra_framework(...)
    public entry fun should_forbid_to_initialize_from_not_owner(attacker: &signer) {
        // No prior setup; attempting to initialize from a non-framework signer must fail.
        bridge::initialize(attacker);
    }

    /// Adding the same token twice should fail.
    #[test]
    #[expected_failure(abort_code = 16)] // E_ASSET_EXISTS
    public entry fun should_forbid_to_add_token_twice() acquires FARegistry {
        let cf = setup_framework_for_tests();
        add_eth_asset(&cf);
        // Second registration of ETH_L1 must revert.
        add_eth_asset(&cf);
    }

    /// Deposit must revert if L1 token is not supported.
    #[test(msig=@0xAAAA, user=@0xBBBB)]
    #[expected_failure(abort_code = 17)] // E_ASSET_UNKNOWN
    public entry fun should_revert_deposit_if_token_is_not_supported(msig: &signer, user: &signer) acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        // Do NOT call add_eth_asset; use an unknown L1 key.
        bridge::execute_deposit(msig, BAD_L1, signer::address_of(user), 100, 1, b"tx-unknown");
    }

    /// Approval must revert if L1 token is not supported (this guards the withdrawal path).
    #[test(msig=@0xAAAA, user=@0xBBBB)]
    #[expected_failure(abort_code = 17)] // E_ASSET_UNKNOWN
    public entry fun should_revert_approval_if_token_is_not_supported(msig: &signer, user: &signer) acquires FARegistry, Admin, Requests, Config {
        let cf = setup_framework_for_tests();
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        // No asset registered; approval must fail on unknown L1.
        bridge::approve_withdrawal(msig, signer::address_of(user), BAD_L1, b"11111111111111111111", 50, 77);
    }

    /// Only the configured multisig can execute deposits.
    #[test(msig=@0xABCD, not_msig=@0xDCBA, to=@0x3333)]
    #[expected_failure(abort_code = 1)] // E_NOT_ADMIN
    public entry fun should_forbid_deposit_not_multisig(msig: &signer, not_msig: &signer, to: &signer) acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        add_eth_asset(&cf);

        // Attempt the deposit from a non-multisig signer => E_NOT_ADMIN.
        bridge::execute_deposit(not_msig, ETH_L1, signer::address_of(to), 10, 9, b"tx-not-msig");
    }

        // Extra L1 keys (must be 20 bytes each)
    const USDC_L1: vector<u8> = b"11111111111111111111";
    const DAI_L1:  vector<u8> = b"22222222222222222222";

    #[test_only]
    fun add_asset_generic(
        cf: &signer,
        l1: vector<u8>,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8
    ) acquires FARegistry {
        bridge::add_asset(cf, l1, name, symbol, decimals, b"", b"");
    }

    #[test(msig=@0x9001, user=@0x9002)]
    public entry fun test_multi_assets_deposit_and_withdraw(msig:&signer, user:&signer)
    acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));

        // Register 3 assets
        add_asset_generic(&cf, ETH_L1,  b"Wrapped Ether", b"WETH", 18);
        add_asset_generic(&cf, USDC_L1, b"USD Coin",      b"USDC", 6);
        add_asset_generic(&cf, DAI_L1,  b"Dai Stable",    b"DAI",  8);

        let u = signer::address_of(user);

        // Baselines
        let b0_eth = bridge::balance_of(ETH_L1,  u);
        let b0_usd = bridge::balance_of(USDC_L1, u);
        let b0_dai = bridge::balance_of(DAI_L1,  u);

        // Deposits (mint)
        bridge::execute_deposit(msig, ETH_L1,  u, 500,  101, b"tx-weth-101");
        bridge::execute_deposit(msig, USDC_L1, u, 1200, 102, b"tx-usdc-102");
        bridge::execute_deposit(msig, DAI_L1,  u, 333,  103, b"tx-dai-103");

        assert!(bridge::balance_of(ETH_L1,  u) == b0_eth + 500,  0x2011);
        assert!(bridge::balance_of(USDC_L1, u) == b0_usd + 1200, 0x2012);
        assert!(bridge::balance_of(DAI_L1,  u) == b0_dai + 333,  0x2013);

        // Approvals (one per asset)
        let recp = b"AAAAAAAAAAAAAAAAAAAA"; // 20 bytes
        bridge::approve_withdrawal(msig, u, ETH_L1,  recp, 200, 201);
        bridge::approve_withdrawal(msig, u, USDC_L1, recp, 300, 202);
        bridge::approve_withdrawal(msig, u, DAI_L1,  recp, 33,  203);

        // User burns (withdraw)
        bridge::withdraw_to_l1(user, ETH_L1,  recp, 200, 201);
        bridge::withdraw_to_l1(user, USDC_L1, recp, 300, 202);
        bridge::withdraw_to_l1(user, DAI_L1,  recp, 33,  203);

        assert!(bridge::balance_of(ETH_L1,  u) == b0_eth + 500 - 200,   0x2021);
        assert!(bridge::balance_of(USDC_L1, u) == b0_usd + 1200 - 300,  0x2022);
        assert!(bridge::balance_of(DAI_L1,  u) == b0_dai + 333 - 33,    0x2023);

        // Nonces consumed
        assert!(bridge::nonce_used(101), 0x2031);
        assert!(bridge::nonce_used(102), 0x2032);
        assert!(bridge::nonce_used(103), 0x2033);
        assert!(bridge::nonce_used(201), 0x2034);
        assert!(bridge::nonce_used(202), 0x2035);
        assert!(bridge::nonce_used(203), 0x2036);
    }

    // New deposit should fail after remove_asset
    #[test(msig=@0x8201, user=@0x8202)]
    #[expected_failure(abort_code = 17)] // E_ASSET_UNKNOWN
    public entry fun test_remove_asset_blocks_deposit(msig:&signer, user:&signer)
    acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        add_asset_generic(&cf, ETH_L1, b"Wrapped Ether", b"WETH", 18);
        bridge::remove_asset(&cf, ETH_L1);
        bridge::execute_deposit(msig, ETH_L1, signer::address_of(user), 10, 999, b"nope");
    }

    // Previously-issued approval fails to execute after remove_asset (metadata lookup now fails)
    #[test(msig=@0x8301, user=@0x8302)]
    #[expected_failure(abort_code = 17)] // E_ASSET_UNKNOWN (on withdraw lookup)
    public entry fun test_remove_asset_breaks_preapproved_withdraw(msig:&signer, user:&signer)
    acquires FARegistry, Admin, BridgeEvents, Requests, Config {
        let cf = setup_framework_for_tests();
        bridge::set_multisig_framework_only(&cf, signer::address_of(msig));
        add_asset_generic(&cf, ETH_L1, b"Wrapped Ether", b"WETH", 18);

        let u = signer::address_of(user);
        bridge::execute_deposit(msig, ETH_L1, u, 1000, 7002, b"fund-2");

        let recp = b"CCCCCCCCCCCCCCCCCCCC";
        bridge::approve_withdrawal(msig, u, ETH_L1, recp, 100, 9002);

        // Remove after approval, before user withdraws
        bridge::remove_asset(&cf, ETH_L1);

        // This will fail because withdraw re-fetches metadata via L1 mapping
        bridge::withdraw_to_l1(user, ETH_L1, recp, 100, 9002);
    }
}