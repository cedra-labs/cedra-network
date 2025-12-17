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
    use cedra_framework::fungible_asset::{
        Self as fa,
        Metadata,
        MintRef,
        BurnRef,
        TransferRef,
        FungibleAsset,
        FungibleStore,
    };
    use cedra_framework::object::{Self as object, Object, ExtendRef};
    use cedra_framework::primary_fungible_store;
    use cedra_framework::dispatchable_fungible_asset;

    /* ===================== Multisig (multisig) ===================== */

    struct Multisig has key {
        multisig_address: address,
    }

    /* ===================== Errors ===================== */

    const E_NOT_ADMIN: u64           = 1;
    const E_BAD_INPUT: u64           = 2;
    const E_NONCE_USED: u64          = 3;
    const E_PAUSED: u64              = 4;
    const E_ZERO_AMOUNT: u64         = 5;
    const E_ALREADY_INITIALIZED: u64 = 6;
    const E_ASSET_EXISTS: u64        = 7;
    const E_ASSET_UNKNOWN: u64       = 8;
    const E_ASSET_MISMATCH: u64      = 9;
    const E_ASSET_BLOCKED: u64       = 10;

    /* ===================== Global config ===================== */

    struct Config has key {
        paused_all: bool,
        pause_deposits: bool,
        pause_withdrawals: bool,
    }

    /* ===================== FA caps and registry (wrapped only) ===================== */

    struct FACaps has store {
        mint: MintRef,
        burn: BurnRef,
        transfer: TransferRef,
    }

    struct WrappedAssetsRegistry has key {
        // origin native token (Ethereum 20 bytes) -> Cedra wrapped FA (Metadata object)
        origin_to_wrapped: Table<vector<u8>, Object<Metadata>>,
        // Cedra wrapped FA (Metadata object) -> origin native token (Ethereum 20 bytes)
        wrapped_to_origin: Table<address, vector<u8>>,
        // Cedra wrapped FA metadata address -> caps (only for wrapped assets we create)
        caps_by_meta: Table<address, FACaps>,
        // explicit lists for viewing
        origin_tokens:     vector<vector<u8>>, // all origin ERC20s
        wrapped_meta_addrs: vector<address>,   // all wrapped Metadata addresses
    }

    /* ===================== Native whitelist ===================== */

    struct NativeWhitelist has key {
        // Cedra-native FA metadata addresses allowed to lock/unlock via bridge
        native_assets: Table<address, bool>,
        // Explicit list so we can expose it in a view
        native_asset_list: vector<address>,
    }

    /* ===================== Blocked assets (native + wrapped) ===================== */

    struct BlockedAssets has key {
        blocked_deposits: Table<address, bool>,
        blocked_withdrawals: Table<address, bool>,
    }

    /* ===================== Native vaults (locking Cedra-native) ===================== */

    #[resource_group_member(group = cedra_framework::object::ObjectGroup)]
    struct NativeVault has key {
        // Used to sign transfers out of the vault store
        extend_ref: ExtendRef,
    }

    /// meta_addr (native FA) -> vault object address (which holds FungibleStore)
    struct NativeVaults has key {
        vaults: Table<address, address>,
    }

    /* ===================== Nonces ===================== */

    struct Requests has key {
        /// Local nonce for Cedra -> remote deposits (id for Deposit events)
        next_deposit_nonce: u64,

        /// Replay-protection for remote -> Cedra withdrawals.
        /// Key = source-chain deposit nonce (u64)
        processed_remote_nonces: Table<u64, bool>,
    }

    /* ===================== View structs ===================== */

    struct SupportedAssetView has drop, store {
        /// Cedra Metadata address of the asset.
        asset: address,
        /// true if this is a Cedra-wrapped asset (for an origin ERC20).
        is_wrapped: bool,
        /// For wrapped assets: origin ERC20 address (20 bytes). None for native.
        origin: option::Option<vector<u8>>,
    }

    /* ===================== Events ===================== */

    /// - user on Cedra deposits (locks or burns) and we signal Ethereum.
    #[event]
    struct Deposit has drop, store {
        asset: address,               // Cedra Metadata address
        from: address,
        remote_recipient: vector<u8>, // 20 bytes Ethereum address
        amount: u64,
        nonce: u64,
    }

    /// - multisig on Cedra mints or unlocks tokens locally.
    #[event]
    struct Withdraw has drop, store {
        asset: address, // Cedra Metadata address
        to: address,
        amount: u64,
        nonce: u64,
    }

    struct BridgeEvents has key {
        deposit: EventHandle<Deposit>,
        withdraw: EventHandle<Withdraw>,
    }

    /* ===================== Internal helpers ===================== */

    inline fun assert_not_paused(is_deposit: bool) acquires Config {
        let cfg = borrow_global<Config>(@bridge);

        // Global kill switch
        assert!(!cfg.paused_all, E_PAUSED);

        if (is_deposit) {
            assert!(!cfg.pause_deposits, E_PAUSED);
        } else {
            assert!(!cfg.pause_withdrawals, E_PAUSED);
        }
    }

    /// Governance / owner authority: must be @bridge_owner.
    inline fun assert_owner(s: &signer) {
        // Only bridge_owner can do certain bootstrap / governance actions.
        assert!(signer::address_of(s) == @bridge, E_NOT_ADMIN);
    }

    inline fun assert_multisig(s: &signer) acquires Multisig {
        let multisig = borrow_global<Multisig>(@bridge);
        assert!(signer::address_of(s) == multisig.multisig_address, E_NOT_ADMIN);
    }

    inline fun assert_20_bytes(addr: &vector<u8>) {
        assert!(vector::length(addr) == 20, E_BAD_INPUT);
    }

    inline fun get_registry(): &mut WrappedAssetsRegistry {
        borrow_global_mut<WrappedAssetsRegistry>(@bridge)
    }

    inline fun get_requests(): &mut Requests {
        borrow_global_mut<Requests>(@bridge)
    }

    inline fun get_whitelist(): &mut NativeWhitelist {
        borrow_global_mut<NativeWhitelist>(@bridge)
    }

    inline fun get_vaults(): &mut NativeVaults {
        borrow_global_mut<NativeVaults>(@bridge)
    }

    inline fun get_blocked_assets(): &mut BlockedAssets {
        borrow_global_mut<BlockedAssets>(@bridge)
    }

    inline fun take_next_deposit_nonce(): u64 acquires Requests {
        let reqs = get_requests();
        let n = reqs.next_deposit_nonce;
        reqs.next_deposit_nonce = n + 1;
        n
    }

    inline fun assert_and_mark_remote_nonce(src_nonce: u64) acquires Requests {
        let reqs = get_requests();
        assert!(
            !table::contains(&reqs.processed_remote_nonces, src_nonce),
            E_NONCE_USED
        );
        table::add(&mut reqs.processed_remote_nonces, src_nonce, true);
    }

    inline fun get_caps_or_abort(meta_addr: address): &mut FACaps acquires WrappedAssetsRegistry {
        let reg = borrow_global_mut<WrappedAssetsRegistry>(@bridge);
        assert!(table::contains(&reg.caps_by_meta, meta_addr), E_ASSET_UNKNOWN);
        table::borrow_mut(&mut reg.caps_by_meta, meta_addr)
    }

    inline fun get_wrapped_meta_for_origin_or_abort(origin_token: &vector<u8>): Object<Metadata> acquires WrappedAssetsRegistry {
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        assert!(table::contains(&reg.origin_to_wrapped, *origin_token), E_ASSET_UNKNOWN);
        *table::borrow(&reg.origin_to_wrapped, *origin_token)
    }

    /* ===================== Init & admin ===================== */

    public entry fun initialize(
        bridge_owner: &signer,
        initial_multisig: address,
    ) {
        assert_owner(bridge_owner);
        assert!(!exists<Config>(@bridge), E_ALREADY_INITIALIZED);

        move_to(bridge_owner, Config {
            paused_all: false,
            pause_deposits: false,
            pause_withdrawals: false,
        });

        move_to(bridge_owner, WrappedAssetsRegistry {
            origin_to_wrapped:   table::new<vector<u8>, Object<Metadata>>(),
            wrapped_to_origin:   table::new<address, vector<u8>>(),
            caps_by_meta:        table::new<address, FACaps>(),
            origin_tokens:       vector::empty<vector<u8>>(),
            wrapped_meta_addrs:  vector::empty<address>(),
        });

        move_to(bridge_owner, NativeWhitelist {
            native_assets:      table::new<address, bool>(),
            native_asset_list:  vector::empty<address>(),
        });

        move_to(bridge_owner, BlockedAssets {
            blocked_deposits:     table::new<address, bool>(),
            blocked_withdrawals:  table::new<address, bool>(),
        });

        move_to(bridge_owner, NativeVaults {
            vaults: table::new<address, address>(),
        });

        move_to(bridge_owner, Requests {
            next_deposit_nonce: 0,
            processed_remote_nonces: table::new<u64, bool>(),
        });

        move_to(bridge_owner, BridgeEvents {
            deposit:  account::new_event_handle<Deposit>(bridge_owner),
            withdraw: account::new_event_handle<Withdraw>(bridge_owner),
        });

        // Initial multisig = passed in address (can be different from bridge_owner).
        move_to(bridge_owner, Multisig { multisig_address: initial_multisig });
    }

    /// Rotate multisig admin.
    /// This is a governance action, so it is restricted to bridge_owner,
    /// not to the existing multisig.
    public entry fun rotate_multisig(
        bridge_owner: &signer,
        new_addr: address,
    ) acquires Multisig {
        assert_owner(bridge_owner);
        borrow_global_mut<Multisig>(@bridge).multisig_address = new_addr;
    }

    public fun pause(bridge_owner: &signer) acquires Config {
        assert_owner(bridge_owner);
        let cfg = borrow_global_mut<Config>(@bridge);
        cfg.paused_all = true;
    }

    public fun unpause(bridge_owner: &signer) acquires Config {
        assert_owner(bridge_owner);
        let cfg = borrow_global_mut<Config>(@bridge);
        cfg.paused_all = false;
        cfg.pause_deposits = false;
        cfg.pause_withdrawals = false;
    }

    public fun pause_deposits(bridge_owner: &signer) acquires Config {
        assert_owner(bridge_owner);
        borrow_global_mut<Config>(@bridge).pause_deposits = true;
    }

    public fun unpause_deposits(bridge_owner: &signer) acquires Config {
        assert_owner(bridge_owner);
        borrow_global_mut<Config>(@bridge).pause_deposits = false;
    }

    public fun pause_withdrawals(bridge_owner: &signer) acquires Config {
        assert_owner(bridge_owner);
        borrow_global_mut<Config>(@bridge).pause_withdrawals = true;
    }

    public fun unpause_withdrawals(bridge_owner: &signer) acquires Config {
        assert_owner(bridge_owner);
        borrow_global_mut<Config>(@bridge).pause_withdrawals = false;
    }

        /// Block both deposits and withdrawals for an asset (native or wrapped).
    public entry fun block_asset(
        bridge_owner: &signer,
        meta_addr: address,
    ) acquires BlockedAssets, WrappedAssetsRegistry, NativeWhitelist {
        assert_owner(bridge_owner);

        // Require that it is either known wrapped or whitelisted native.
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        let is_wrapped = table::contains(&reg.caps_by_meta, meta_addr);

        let wl = borrow_global<NativeWhitelist>(@bridge);
        let is_native = table::contains(&wl.native_assets, meta_addr);

        assert!(is_wrapped || is_native, E_ASSET_UNKNOWN);

        let bl = get_blocked_assets();
        assert!(!table::contains(&bl.blocked_deposits, meta_addr), E_ASSET_EXISTS);
        assert!(!table::contains(&bl.blocked_withdrawals, meta_addr), E_ASSET_EXISTS);

        table::add(&mut bl.blocked_deposits, meta_addr, true);
        table::add(&mut bl.blocked_withdrawals, meta_addr, true);
    }

    /// Unblock both deposits and withdrawals for an asset.
    public entry fun unblock_asset(
        bridge_owner: &signer,
        meta_addr: address,
    ) acquires BlockedAssets {
        assert_owner(bridge_owner);

        let bl = get_blocked_assets();

        let had_deposits = table::contains(&bl.blocked_deposits, meta_addr);
        let had_withdrawals = table::contains(&bl.blocked_withdrawals, meta_addr);

        // Must be blocked in at least one direction.
        assert!(had_deposits || had_withdrawals, E_ASSET_UNKNOWN);

        if (had_deposits) {
            let _ = table::remove(&mut bl.blocked_deposits, meta_addr);
        };
        if (had_withdrawals) {
            let _ = table::remove(&mut bl.blocked_withdrawals, meta_addr);
        };
    }

    /// Unblock only deposits for an asset.
    public entry fun unblock_asset_deposits(
        bridge_owner: &signer,
        meta_addr: address,
    ) acquires BlockedAssets {
        assert_owner(bridge_owner);

        let bl = get_blocked_assets();
        assert!(table::contains(&bl.blocked_deposits, meta_addr), E_ASSET_UNKNOWN);
        let _ = table::remove(&mut bl.blocked_deposits, meta_addr);
    }

    /// Unblock only withdrawals for an asset.
    public entry fun unblock_asset_withdrawals(
        bridge_owner: &signer,
        meta_addr: address,
    ) acquires BlockedAssets {
        assert_owner(bridge_owner);

        let bl = get_blocked_assets();
        assert!(table::contains(&bl.blocked_withdrawals, meta_addr), E_ASSET_UNKNOWN);
        let _ = table::remove(&mut bl.blocked_withdrawals, meta_addr);
    }

    /* ===================== Registry: wrapped assets only ===================== */

    /// INTERNAL ONLY: create a new Cedra-wrapped asset for an *origin* ERC20
    /// and record it in the registry.
    fun create_wrapped_asset(
        multisig: &signer,
        origin_token: &vector<u8>,
        name: &vector<u8>,
        symbol: &vector<u8>,
        decimals: u8,
        icon_uri: &vector<u8>,
        project_uri: &vector<u8>,
    ): Object<Metadata> acquires WrappedAssetsRegistry, Multisig {
        assert_multisig(multisig);
        assert_20_bytes(origin_token);

        let reg = get_registry();

        assert!(
            !table::contains(&reg.origin_to_wrapped, *origin_token),
            E_ASSET_EXISTS
        );

        // origin_token, name, symbol, icon_uri, project_uri are &vector<u8>
        let ctor = &object::create_named_object(multisig, *origin_token);

        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            ctor,
            // no hard cap
            option::none<u128>(),
            string::utf8(*name),
            string::utf8(*symbol),
            decimals,
            string::utf8(*icon_uri),
            string::utf8(*project_uri),
        );

        let meta: Object<Metadata> = object::object_from_constructor_ref<Metadata>(ctor);
        let mint_ref     = fa::generate_mint_ref(ctor);
        let transfer_ref = fa::generate_transfer_ref(ctor);
        let burn_ref     = fa::generate_burn_ref(ctor);

        let meta_addr = object::object_address(&meta);

        table::add(&mut reg.origin_to_wrapped, *origin_token, meta);
        table::add(
            &mut reg.caps_by_meta,
            meta_addr,
            FACaps { mint: mint_ref, burn: burn_ref, transfer: transfer_ref },
        );
        table::add(
            &mut reg.wrapped_to_origin,
            meta_addr,
            *origin_token,
        );

        vector::push_back(&mut reg.origin_tokens, *origin_token);
        vector::push_back(&mut reg.wrapped_meta_addrs, meta_addr);

        meta
    }

    /* ===================== Native whitelist (no mapping to ETH) ===================== */

    /// Multisig-only: add a Cedra-native FA metadata address to the whitelist.
    public entry fun whitelist_native_token(
        bridge_owner: &signer,
        native_meta_addr: address,
    ) acquires NativeWhitelist, WrappedAssetsRegistry {
        assert_owner(bridge_owner);

        // Must NOT be a wrapped asset: wrapped assets have caps_by_meta entries.
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        assert!(
            !table::contains(&reg.caps_by_meta, native_meta_addr),
            E_ASSET_MISMATCH
        );

        let wl = get_whitelist();
        assert!(
            !table::contains(&wl.native_assets, native_meta_addr),
            E_ASSET_EXISTS
        );

        table::add(&mut wl.native_assets, native_meta_addr, true);
        vector::push_back(&mut wl.native_asset_list, native_meta_addr);
    }

    /// Multisig-only: remove from native whitelist.
    public entry fun unwhitelist_native_token(
        bridge_owner: &signer,
        native_meta_addr: address,
    ) acquires NativeWhitelist {
        assert_owner(bridge_owner);

        let wl = get_whitelist();
        assert!(
            table::contains(&wl.native_assets, native_meta_addr),
            E_ASSET_UNKNOWN
        );
        let _ = table::remove(&mut wl.native_assets, native_meta_addr);

        // Remove from vector (linear search)
        let len = vector::length(&wl.native_asset_list);
        let i = 0;
        while (i < len) {
            let addr = *vector::borrow(&wl.native_asset_list, i);
            if (addr == native_meta_addr) {
                vector::swap_remove(&mut wl.native_asset_list, i);
                break;
            };
            i = i + 1;
        };
    }

    /* ===================== Internal: get or create native vault ===================== */

    inline fun ensure_vault_for_native(
        native_meta_addr: address,
        meta: Object<Metadata>,
    ): address acquires NativeVaults, NativeVault {
        let vaults = get_vaults();
        if (table::contains(&vaults.vaults, native_meta_addr)) {
            *table::borrow(&vaults.vaults, native_meta_addr)
        } else {
            // Create new vault object at @bridge
            let ctor = object::create_object(@bridge);
            let extend_ref = object::generate_extend_ref(&ctor);
            let obj_signer = object::generate_signer(&ctor);

            // Attach a FungibleStore for this FA
            fa::create_store(&ctor, meta);

            // Store NativeVault resource in that object
            move_to(&obj_signer, NativeVault { extend_ref });

            let vault_addr = object::address_from_constructor_ref(&ctor);
            table::add(&mut vaults.vaults, native_meta_addr, vault_addr);
            vault_addr
        }
    }

    /* ===================== Deposits (Cedra -> Ethereum) ===================== */

    /// deposit_native_tokens:
    /// - user deposits a Cedra-native asset;
    /// - tokens are *locked* in a NativeVault (FungibleStore under @bridge);
    /// - emits Deposit(...) event for relayer.
    public entry fun deposit_native_tokens(
        user: &signer,
        native_meta_addr: address,
        remote_recipient: vector<u8>, // Ethereum address
        amount: u64,
    ) acquires WrappedAssetsRegistry, NativeWhitelist, NativeVaults, Requests, BridgeEvents, Config, BlockedAssets {
        assert_not_paused(true);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&remote_recipient);

        // Must be whitelisted & NOT wrapped
        let wl = borrow_global<NativeWhitelist>(@bridge);
        assert!(table::contains(&wl.native_assets, native_meta_addr), E_ASSET_MISMATCH);

        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        assert!(
            !table::contains(&reg.caps_by_meta, native_meta_addr),
            E_ASSET_MISMATCH
        );
        {
            let bl = borrow_global<BlockedAssets>(@bridge);
            assert!(
                !table::contains(&bl.blocked_deposits, native_meta_addr),
                E_ASSET_BLOCKED
            );
        };

        let meta: Object<Metadata> = object::address_to_object<Metadata>(native_meta_addr);

        // Ensure / get vault for this native asset
        let vault_addr = ensure_vault_for_native(native_meta_addr, meta);

        // Transfer from user's primary store into vault store
        let user_addr = signer::address_of(user);
        let user_store = primary_fungible_store::primary_store_inlined(user_addr, meta);
        let vault_store = object::address_to_object<FungibleStore>(vault_addr);

        dispatchable_fungible_asset::transfer(
            user,
            user_store,
            vault_store,
            amount,
        );

        let nonce = take_next_deposit_nonce();

        // Emit deposit event
        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(Deposit {
                asset: native_meta_addr,
                from: user_addr,
                remote_recipient,
                amount,
                nonce,
            });
        } else {
            ev::emit_event(
                &mut evs.deposit,
                Deposit {
                    asset: native_meta_addr,
                    from: user_addr,
                    remote_recipient,
                    amount,
                    nonce,
                },
            );
        };
    }

    /// deposit_wrapped_tokens:
    /// - user deposits a Cedra-wrapped asset (Cedra representation of origin ERC20);
    /// - bridge pulls tokens from user and *burns* them;
    /// - emits Deposit(...) for relayer (like depositWrapped on ETH).
    public entry fun deposit_wrapped_tokens(
        user: &signer,
        wrapped_meta_addr: address,
        remote_recipient: vector<u8>, // Ethereum address
        amount: u64,
    ) acquires WrappedAssetsRegistry, Requests, BridgeEvents, Config, BlockedAssets {
        assert_not_paused(true);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&remote_recipient);

        // Must be a known wrapped asset (has caps)
        {
            let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
            assert!(
                table::contains(&reg.caps_by_meta, wrapped_meta_addr),
                E_ASSET_MISMATCH
            );
        };
        // Must not be blocked on deposit path
        {
            let bl = borrow_global<BlockedAssets>(@bridge);
            assert!(
                !table::contains(&bl.blocked_deposits, wrapped_meta_addr),
                E_ASSET_BLOCKED
            );
        };

        let meta: Object<Metadata> = object::address_to_object<Metadata>(wrapped_meta_addr);
        let withdrawn: FungibleAsset = primary_fungible_store::withdraw<Metadata>(user, meta, amount);

        let caps = get_caps_or_abort(wrapped_meta_addr);
        fa::burn(&caps.burn, withdrawn);

        let nonce = take_next_deposit_nonce();

        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(Deposit {
                asset: wrapped_meta_addr,
                from: signer::address_of(user),
                remote_recipient,
                amount,
                nonce,
            });
        } else {
            ev::emit_event(
                &mut evs.deposit,
                Deposit {
                    asset: wrapped_meta_addr,
                    from: signer::address_of(user),
                    remote_recipient,
                    amount,
                    nonce,
                },
            );
        };
    }

    /* ===================== Withdrawals (Ethereum -> Cedra) ===================== */

    /// withdraw_tokens:
    /// - called by multisig when processing a deposit on Ethereum;
    /// - if asset is wrapped: mint on Cedra and send to `to`;
    /// - if asset is native: unlock from vault and send to `to`.
    public entry fun withdraw_tokens(
        multisig: &signer,
        meta_addr: address,
        to: address,
        amount: u64,
        src_nonce: u64,
    ) acquires
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Config,
        Multisig,
        BlockedAssets {
        assert_not_paused(false);
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);

        // replay protection: remote chain + remote deposit nonce
        assert_and_mark_remote_nonce(src_nonce);

        // Block any withdrawals for this asset if configured
        {
            let bl = borrow_global<BlockedAssets>(@bridge);
            assert!(
                !table::contains(&bl.blocked_withdrawals, meta_addr),
                E_ASSET_BLOCKED
            );
        };

        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        let is_wrapped = table::contains(&reg.caps_by_meta, meta_addr);

        if (is_wrapped) {
            // Cedra-wrapped asset: mint
            let meta: Object<Metadata> = object::address_to_object<Metadata>(meta_addr);
            let caps = get_caps_or_abort(meta_addr);
            let minted: FungibleAsset = fa::mint(&caps.mint, amount);
            primary_fungible_store::deposit(to, minted);
        } else {
            // Cedra-native asset: must be whitelisted, then unlock from vault and send to `to`.
            let wl = borrow_global<NativeWhitelist>(@bridge);
            assert!(table::contains(&wl.native_assets, meta_addr), E_ASSET_UNKNOWN);

            let vaults = borrow_global<NativeVaults>(@bridge);
            assert!(table::contains(&vaults.vaults, meta_addr), E_ASSET_UNKNOWN);
            let vault_addr = *table::borrow(&vaults.vaults, meta_addr);

            let meta: Object<Metadata> = object::address_to_object<Metadata>(meta_addr);
            let vault_store = object::address_to_object<FungibleStore>(vault_addr);
            let to_store = primary_fungible_store::ensure_primary_store_exists(to, meta);

            let vault_res = &NativeVault[vault_addr];
            let vault_signer = object::generate_signer_for_extending(&vault_res.extend_ref);

            dispatchable_fungible_asset::transfer(
                &vault_signer,
                vault_store,
                to_store,
                amount,
            );
        };

        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(Withdraw { asset: meta_addr, to, amount, nonce: src_nonce });
        } else {
            ev::emit_event(&mut evs.withdraw, Withdraw { asset: meta_addr, to, amount, nonce: src_nonce });
        };
    }

    /// withdraw_auto_create_wrapped:
    /// - called by multisig when seeing a *new* origin asset on Ethereum;
    /// - if no Cedra-wrapped asset exists for `origin_token`, create it via
    ///   `create_wrapped_asset` and then mint to `to`.
    public entry fun withdraw_auto_create_wrapped(
        multisig: &signer,
        origin_token: vector<u8>,   // origin native token on Ethereum (20 bytes)
        to: address,
        amount: u64,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
        icon_uri: vector<u8>,
        project_uri: vector<u8>,
        src_nonce: u64,
    ) acquires WrappedAssetsRegistry, Requests, BridgeEvents, Config, Multisig, BlockedAssets {
        assert_not_paused(false);
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&origin_token);

        // replay protection: remote chain + remote deposit nonce
        assert_and_mark_remote_nonce(src_nonce);

        let reg = get_registry();
        let wrapped_meta: Object<Metadata>;

        if (table::contains(&reg.origin_to_wrapped, origin_token)) {
            wrapped_meta = *table::borrow(&reg.origin_to_wrapped, origin_token);
        } else {
            // Auto-create wrapped FA for this origin asset.
            wrapped_meta = create_wrapped_asset(
                multisig,
                &origin_token,
                &name,
                &symbol,
                decimals,
                &icon_uri,
                &project_uri,
            );
        };

        let wrapped_meta_addr = object::object_address(&wrapped_meta);
        {
            let bl = borrow_global<BlockedAssets>(@bridge);
            assert!(
                !table::contains(&bl.blocked_withdrawals, wrapped_meta_addr),
                E_ASSET_BLOCKED
            );
        };
        let caps = get_caps_or_abort(wrapped_meta_addr);
        let minted: FungibleAsset = fa::mint(&caps.mint, amount);
        primary_fungible_store::deposit(to, minted);

        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(Withdraw { asset: wrapped_meta_addr, to, amount, nonce: src_nonce });
        } else {
            ev::emit_event(
                &mut evs.withdraw,
                Withdraw { asset: wrapped_meta_addr, to, amount, nonce: src_nonce },
            );
        };
    }

    /* ===================== Views / helpers ===================== */

    #[view]
    public fun admin_multisig(): address acquires Multisig {
        borrow_global<Multisig>(@bridge).multisig_address
    }

    #[view]
    public fun next_deposit_nonce(): u64 acquires Requests {
        borrow_global<Requests>(@bridge).next_deposit_nonce
    }

    #[view]
    public fun remote_nonce_processed(src_nonce: u64): bool acquires Requests {
        let reqs = borrow_global<Requests>(@bridge);
        table::contains(&reqs.processed_remote_nonces, src_nonce)
    }

    /// Check if a Cedra asset (by Metadata address) is wrapped.
    /// Logic: wrapped assets are exactly those with caps in `caps_by_meta`.
    #[view]
    public fun is_wrapped_asset(meta_addr: address): bool acquires WrappedAssetsRegistry {
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        table::contains(&reg.caps_by_meta, meta_addr)
    }

    /// Check if a Cedra asset (by Metadata address) is native-whitelisted.
    #[view]
    public fun is_native_asset(meta_addr: address): bool acquires NativeWhitelist {
        let wl = borrow_global<NativeWhitelist>(@bridge);
        table::contains(&wl.native_assets, meta_addr)
    }

    /// Return the Cedra wrapped asset (Metadata address) for a given origin token.
    #[view]
    public fun wrapped_meta_of_origin(origin_token: vector<u8>): address acquires WrappedAssetsRegistry {
        let meta = get_wrapped_meta_for_origin_or_abort(&origin_token);
        object::object_address(&meta)
    }

    #[view]
    public fun origin_of_wrapped(meta_addr: address): vector<u8> acquires WrappedAssetsRegistry {
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        assert!(table::contains(&reg.wrapped_to_origin, meta_addr), E_ASSET_UNKNOWN);
        *table::borrow(&reg.wrapped_to_origin, meta_addr)
    }

    #[view]
    public fun all_wrapped_meta_addrs(): vector<address> acquires WrappedAssetsRegistry {
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        reg.wrapped_meta_addrs
    }

    #[view]
    public fun all_origin_tokens(): vector<vector<u8>> acquires WrappedAssetsRegistry {
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        reg.origin_tokens
    }

    #[view]
    public fun all_native_whitelisted_assets(): vector<address> acquires NativeWhitelist {
        let wl = borrow_global<NativeWhitelist>(@bridge);
        wl.native_asset_list
    }

    #[view]
    public fun native_vault_address(
        meta_addr: address
    ): option::Option<address> acquires NativeVaults {
        let vaults = borrow_global<NativeVaults>(@bridge);
        if (table::contains(&vaults.vaults, meta_addr)) {
            option::some(*table::borrow(&vaults.vaults, meta_addr))
        } else {
            option::none<address>()
        }
    }

    #[view]
    public fun native_vault_balance(
        meta_addr: address
    ): u64 acquires NativeVaults {
        let vaults = borrow_global<NativeVaults>(@bridge);
        assert!(table::contains(&vaults.vaults, meta_addr), E_ASSET_UNKNOWN);
        let vault_addr = *table::borrow(&vaults.vaults, meta_addr);

        let store = object::address_to_object<FungibleStore>(vault_addr);
        fa::balance(store)
    }

    #[view]
    public fun is_asset_deposit_blocked(meta_addr: address): bool acquires BlockedAssets {
        let bl = borrow_global<BlockedAssets>(@bridge);
        table::contains(&bl.blocked_deposits, meta_addr)
    }

    #[view]
    public fun is_asset_withdrawal_blocked(meta_addr: address): bool acquires BlockedAssets {
        let bl = borrow_global<BlockedAssets>(@bridge);
        table::contains(&bl.blocked_withdrawals, meta_addr)
    }

        /// All supported assets on Cedra:
    /// - whitelisted native assets
    /// - all wrapped assets
    ///
    /// For each asset we return:
    /// - asset: Cedra Metadata address
    /// - is_wrapped: true/false
    /// - origin: 20-byte origin token for wrapped; none for native
    #[view]
    public fun all_supported_assets_detailed(): vector<SupportedAssetView>
    acquires NativeWhitelist, WrappedAssetsRegistry {
        let wl = borrow_global<NativeWhitelist>(@bridge);
        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);

        let natives = &wl.native_asset_list;
        let wrapped = &reg.wrapped_meta_addrs;

        let out = vector::empty<SupportedAssetView>();

        // 1) Native-whitelisted assets
        let i = 0;
        let natives_len = vector::length(natives);
        while (i < natives_len) {
            let meta_addr = *vector::borrow(natives, i);
            vector::push_back(
                &mut out,
                SupportedAssetView {
                    asset:      meta_addr,
                    is_wrapped: false,
                    origin:     option::none<vector<u8>>(),
                },
            );
            i = i + 1;
        };

        // 2) Wrapped assets (with origin token filled)
        let j = 0;
        let wrapped_len = vector::length(wrapped);
        while (j < wrapped_len) {
            let meta_addr = *vector::borrow(wrapped, j);
            // must exist for wrapped_meta_addrs entries
            let origin = *table::borrow(&reg.wrapped_to_origin, meta_addr);

            vector::push_back(
                &mut out,
                SupportedAssetView {
                    asset:      meta_addr,
                    is_wrapped: true,
                    origin:     option::some(origin),
                },
            );
            j = j + 1;
        };

        out
    }
    
    /* ===================== UX helper ===================== */

    /// No-op with FA: primary store is auto-created on demand. Left for CLI parity.
    public entry fun ensure_store(_user: &signer) {}

        /* ===================== TESTS ===================== */

    #[test_only]
    fun setup_bridge_and_native_fa_for_test(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ): (Object<Metadata>, address, address) {
        // initialize bridge once
        initialize(bridge_owner, signer::address_of(bridge_owner));

        // Create a test FA as Cedra-native asset
        let (creator_ref, metadata) = fa::create_test_token(asset_admin);
        let (mint_ref, _transfer_ref, _burn_ref) =
            primary_fungible_store::init_test_metadata_with_primary_store_enabled(&creator_ref);

        let user_addr = signer::address_of(user);
        // Mint 100 units to user (matches Cedra's test pattern)
        let initial_mint = 100;
        primary_fungible_store::mint(&mint_ref, user_addr, initial_mint);

        let meta: Object<Metadata> = object::convert(metadata);
        let meta_addr = object::object_address(&meta);

        (meta, meta_addr, user_addr)
    }

    #[test_only]
    fun setup_bridge_and_wrapped_fa_for_test(
        bridge_owner: &signer,
        user: &signer,
    ): (Object<Metadata>, address, address) acquires WrappedAssetsRegistry, Multisig {
        initialize(bridge_owner, signer::address_of(bridge_owner));
        let user_addr = signer::address_of(user);

        // Example origin ERC20 (20 bytes)
        let origin_token = vector[
            0x01u8, 0x02u8, 0x03u8, 0x04u8, 0x05u8,
            0x06u8, 0x07u8, 0x08u8, 0x09u8, 0x0Au8,
            0x0Bu8, 0x0Cu8, 0x0Du8, 0x0Eu8, 0x0Fu8,
            0x10u8, 0x11u8, 0x12u8, 0x13u8, 0x14u8
        ];

        let wrapped_meta = create_wrapped_asset(
            bridge_owner,
            &origin_token,
            &b"TestWrapped",
            &b"TST",
            6,
            &b"",
            &b"",
        );
        let wrapped_meta_addr = object::object_address(&wrapped_meta);

        // Mint 100 wrapped tokens to user
        let caps = get_caps_or_abort(wrapped_meta_addr);
        let minted: FungibleAsset = fa::mint(&caps.mint, 100);
        primary_fungible_store::deposit(user_addr, minted);

        (wrapped_meta, wrapped_meta_addr, user_addr)
    }

    #[test_only]
    fun dummy_remote_recipient(): vector<u8> {
        vector[
            1u8, 2u8, 3u8, 4u8, 5u8,
            6u8, 7u8, 8u8, 9u8, 10u8,
            11u8, 12u8, 13u8, 14u8, 15u8,
            16u8, 17u8, 18u8, 19u8, 20u8
        ]
    }

    /// Native path:
    /// - whitelist native token
    /// - deposit_native_tokens locks into vault
    /// - withdraw_tokens unlocks from vault to another user
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    fun test_deposit_native_and_withdraw(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (meta, meta_addr, user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        assert!(admin_multisig() == signer::address_of(bridge_owner));

        whitelist_native_token(bridge_owner, meta_addr);
        assert!(is_native_asset(meta_addr));

        let before_user = primary_fungible_store::balance(user_addr, meta);

        deposit_native_tokens(user, meta_addr, dummy_remote_recipient(), 60);

        // Cedra deposit nonce increments (0 used in event, next becomes 1)
        assert!(next_deposit_nonce() == 1);

        let after_user = primary_fungible_store::balance(user_addr, meta);
        assert!(after_user == before_user - 60);

        let vaults = borrow_global<NativeVaults>(@bridge);
        let vault_addr = *table::borrow(&vaults.vaults, meta_addr);
        let vault_store = object::address_to_object<FungibleStore>(vault_addr);
        assert!(fa::balance(vault_store) == 60);

        // remote deposit nonce (from Ethereum/Arb) we are processing on Cedra:
        let src_nonce = 777;

        assert!(!remote_nonce_processed(src_nonce));

        withdraw_tokens(bridge_owner, meta_addr, user2_addr, 25, src_nonce);

        assert!(remote_nonce_processed(src_nonce));

        let vault_after = fa::balance(vault_store);
        let user2_balance = primary_fungible_store::balance(user2_addr, meta);

        assert!(vault_after == 35);
        assert!(user2_balance == 25);
    }

    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    #[expected_failure(abort_code = E_NONCE_USED, location = bridge::bridge)]
    fun test_withdraw_replay_protection(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (_meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);

        whitelist_native_token(bridge_owner, meta_addr);
        deposit_native_tokens(user, meta_addr, dummy_remote_recipient(), 40);

        let to = signer::address_of(user2);
        let src_nonce = 42;

        withdraw_tokens(bridge_owner, meta_addr, to, 10, src_nonce);
        // second time with same src_nonce must abort
        withdraw_tokens(bridge_owner, meta_addr, to, 10, src_nonce);
    }

    /// Wrapped auto-create path:
    /// - withdraw_auto_create_wrapped mints wrapped tokens for a new origin
    /// - user can transfer wrapped tokens to another account
    /// - second call reuses same wrapped FA
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    fun test_withdraw_auto_create_wrapped_creates_and_reuses(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        NativeWhitelist,
        BlockedAssets,
    {
        initialize(bridge_owner, signer::address_of(bridge_owner));
        let user_addr = signer::address_of(user);

        // Example origin ERC20 (20 bytes)
        let origin_token = vector[
            0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8, 0xEEu8,
            0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8,
            0x66u8, 0x77u8, 0x88u8, 0x99u8, 0xAAu8,
            0xBBu8, 0xCCu8, 0xDDu8, 0xEEu8, 0xFFu8
        ];

        // First withdraw: should create wrapped FA and mint 500
        let src_nonce_1 = 1000;
        assert!(!remote_nonce_processed(src_nonce_1));

        withdraw_auto_create_wrapped(
            bridge_owner,
            copy origin_token,
            user_addr,
            500,
            b"CedraWrapped",
            b"cTOK",
            6,
            b"",
            b"",
            src_nonce_1,
        );

        assert!(remote_nonce_processed(src_nonce_1));

        let reg = borrow_global<WrappedAssetsRegistry>(@bridge);
        assert!(table::contains(&reg.origin_to_wrapped, origin_token));
        let wrapped_meta_1 = *table::borrow(&reg.origin_to_wrapped, origin_token);
        let wrapped_meta_addr_1 = object::object_address(&wrapped_meta_1);

        assert!(is_wrapped_asset(wrapped_meta_addr_1));
        assert!(!is_native_asset(wrapped_meta_addr_1));

        // User should have 500 wrapped
        assert!(primary_fungible_store::balance(user_addr, wrapped_meta_1) == 500);

        // User transfers part of wrapped tokens to another address
        let recipient_addr = @0xBEEF;

        let before_user     = primary_fungible_store::balance(user_addr, wrapped_meta_1);
        let before_recipient = primary_fungible_store::balance(recipient_addr, wrapped_meta_1);

        let transfer_amount = 200;

        primary_fungible_store::transfer<Metadata>(
            user,
            wrapped_meta_1,
            recipient_addr,
            transfer_amount,
        );

        let after_user      = primary_fungible_store::balance(user_addr, wrapped_meta_1);
        let after_recipient = primary_fungible_store::balance(recipient_addr, wrapped_meta_1);

        assert!(after_user == before_user - transfer_amount);
        assert!(after_recipient == before_recipient + transfer_amount);

        // Second withdraw: should reuse same wrapped asset and mint +100
        let src_nonce_2 = 1001;
        withdraw_auto_create_wrapped(
            bridge_owner,
            copy origin_token,
            user_addr,
            100,
            b"IGNORED",
            b"IGN",
            6,
            b"",
            b"",
            src_nonce_2,
        );
        assert!(remote_nonce_processed(src_nonce_2));

        let reg2 = borrow_global<WrappedAssetsRegistry>(@bridge);
        let wrapped_meta_2 = *table::borrow(&reg2.origin_to_wrapped, origin_token);
        let wrapped_meta_addr_2 = object::object_address(&wrapped_meta_2);

        // Must point to same Metadata object
        assert!(wrapped_meta_addr_2 == wrapped_meta_addr_1);

        // original 500 - 200 transfer + 100 second withdraw = 400 on user
        assert!(primary_fungible_store::balance(user_addr, wrapped_meta_2) == 400);

        let via_view = wrapped_meta_of_origin(origin_token);
        assert!(via_view == wrapped_meta_addr_1);
    }

    /// deposit_wrapped_tokens:
    /// - create wrapped asset
    /// - mint to user
    /// - deposit_wrapped_tokens burns user balance
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    fun test_deposit_wrapped_burns(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (wrapped_meta, wrapped_meta_addr, user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        assert!(primary_fungible_store::balance(user_addr, wrapped_meta) == 100);
        assert!(is_wrapped_asset(wrapped_meta_addr));
        assert!(!is_native_asset(wrapped_meta_addr));

        let remote_recipient = dummy_remote_recipient();

        deposit_wrapped_tokens(
            user,
            wrapped_meta_addr,
            remote_recipient,
            100,
        );

        assert!(next_deposit_nonce() == 1);
        assert!(primary_fungible_store::balance(user_addr, wrapped_meta) == 0);
    }

    /* ========== PAUSE TESTS ========== */

    /// Global pause blocks native deposits.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_pause_block_native_deposits(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        Requests,
        BridgeEvents,
        BlockedAssets
    {
        let (_meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);

        whitelist_native_token(bridge_owner, meta_addr);
        pause(bridge_owner);

        let remote_recipient = dummy_remote_recipient();

        // Must abort with E_PAUSED
        deposit_native_tokens(
            user,
            meta_addr,
            remote_recipient,
            10,
        );
    }

    /// Global pause blocks wrapped deposits.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_pause_block_wrapped_deposits(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (_wrapped_meta, wrapped_meta_addr, _user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        pause(bridge_owner);

        let remote_recipient = dummy_remote_recipient();

        deposit_wrapped_tokens(
            user,
            wrapped_meta_addr,
            remote_recipient,
            10,
        );
    }

    /// pause_withdrawals blocks native withdrawals.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_pause_block_native_withdrawal(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (_meta, meta_addr, user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);

        whitelist_native_token(bridge_owner, meta_addr);

        // Seed vault so withdraw path is reachable
        deposit_native_tokens(
            user,
            meta_addr,
            dummy_remote_recipient(),
            20,
        );

        // Only withdrawals are paused, deposits are still allowed.
        pause_withdrawals(bridge_owner);

        // Native withdraw must now abort with E_PAUSED
        let src_nonce = 777;
        withdraw_tokens(
            bridge_owner,
            meta_addr,
            user_addr,
            10,
            src_nonce
        );
    }

    /// pause_withdrawals blocks wrapped withdrawals.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_pause_block_wrapped_withdrawal(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeVault,
        NativeVaults,
        NativeWhitelist,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (_wrapped_meta, wrapped_meta_addr, user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        // Only withdrawals are paused, deposits still allowed.
        pause_withdrawals(bridge_owner);

        // Wrapped withdraw must now abort with E_PAUSED
        let src_nonce = 777;
        withdraw_tokens(
            bridge_owner,
            wrapped_meta_addr,
            user_addr,
            10,
            src_nonce
        );
    }

    /// pause_deposits blocks deposits but not withdrawals.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    fun test_pause_block_deposits_does_not_block_withdrawals(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (meta, meta_addr, user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        whitelist_native_token(bridge_owner, meta_addr);

        // Initial deposit to seed vault
        let remote_recipient = dummy_remote_recipient();
        deposit_native_tokens(user, meta_addr, remote_recipient, 40);

        pause_deposits(bridge_owner);

        let cfg = borrow_global<Config>(@bridge);
        assert!(!cfg.paused_all);
        assert!(cfg.pause_deposits);
        assert!(!cfg.pause_withdrawals);

        // Withdraw should still work
        let src_nonce = 777;
        withdraw_tokens(
            bridge_owner,
            meta_addr,
            user2_addr,
            10,
            src_nonce
        );

        let vaults = borrow_global<NativeVaults>(@bridge);
        let vault_addr = *table::borrow(&vaults.vaults, meta_addr);
        let vault_store = object::address_to_object<FungibleStore>(vault_addr);
        let vault_balance = fa::balance(vault_store);
        let user2_balance = primary_fungible_store::balance(user2_addr, meta);

        assert!(vault_balance == 30);
        assert!(user2_balance == 10);
        // We deliberately do NOT try a second deposit here (it would abort).
    }

    /// pause_withdrawals blocks withdrawals but not deposits.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE)]
    fun test_pause_block_withdrawals_does_not_block_deposits(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        Requests,
        BridgeEvents,
        BlockedAssets
    {
        let (_meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);

        whitelist_native_token(bridge_owner, meta_addr);

        pause_withdrawals(bridge_owner);

        let cfg = borrow_global<Config>(@bridge);
        assert!(!cfg.paused_all);
        assert!(!cfg.pause_deposits);
        assert!(cfg.pause_withdrawals);

        let remote_recipient = dummy_remote_recipient();

        // Deposit should still work while only withdrawals are paused.
        deposit_native_tokens(
            user,
            meta_addr,
            remote_recipient,
            20,
        );
    }

    /// pause + unpause restores both deposits and withdrawals.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    fun test_unpause_deposits_and_withdrawals(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (meta, meta_addr, user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        whitelist_native_token(bridge_owner, meta_addr);

        let remote_recipient = dummy_remote_recipient();
        deposit_native_tokens(user, meta_addr, remote_recipient, 30);

        pause(bridge_owner);
        unpause(bridge_owner);

        let cfg = borrow_global<Config>(@bridge);
        assert!(!cfg.paused_all);
        assert!(!cfg.pause_deposits);
        assert!(!cfg.pause_withdrawals);

        // After unpause, both deposit and withdraw should work.
        deposit_native_tokens(user, meta_addr, dummy_remote_recipient(), 10);

        let src_nonce = 777;
        withdraw_tokens(bridge_owner, meta_addr, user2_addr, 20, src_nonce);

        let vaults = borrow_global<NativeVaults>(@bridge);
        let vault_addr = *table::borrow(&vaults.vaults, meta_addr);
        let vault_store = object::address_to_object<FungibleStore>(vault_addr);
        let vault_balance = fa::balance(vault_store);

        let user_balance = primary_fungible_store::balance(user_addr, meta);
        let user2_balance = primary_fungible_store::balance(user2_addr, meta);

        assert!(vault_balance + user_balance + user2_balance == 100);
    }

    /// pause + unpause_deposits alone does NOT unblock withdrawals (global kill switch still on).
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_unpause_deposits_does_not_unblock_withdrawals(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        whitelist_native_token(bridge_owner, meta_addr);

        let remote_recipient = dummy_remote_recipient();
        deposit_native_tokens(user, meta_addr, remote_recipient, 40);

        pause(bridge_owner);
        unpause_deposits(bridge_owner);

        // Withdraw is still blocked by paused_all
        let src_nonce = 777;
        withdraw_tokens(
            bridge_owner,
            meta_addr,
            user2_addr,
            10,
            src_nonce
        );
    }

    /// pause + unpause_withdrawals alone does NOT unblock deposits.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_unpause_withdrawals_does_not_unblock_deposits(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        Requests,
        BridgeEvents,
        BlockedAssets
    {
        let (_meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);

        whitelist_native_token(bridge_owner, meta_addr);
        pause(bridge_owner);
        unpause_withdrawals(bridge_owner);

        let remote_recipient = dummy_remote_recipient();

        // Deposit is still blocked by paused_all
        deposit_native_tokens(
            user,
            meta_addr,
            remote_recipient,
            10,
        );
    }

    /* ========== BLOCK / UNBLOCK TESTS ========== */

    /// block_asset blocks native deposits.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE)]
    #[expected_failure(abort_code = E_ASSET_BLOCKED, location = bridge::bridge)]
    fun test_block_asset_blocks_native_deposits(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        Requests,
        BridgeEvents,
        BlockedAssets
    {
        let (_meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);

        whitelist_native_token(bridge_owner, meta_addr);
        block_asset(bridge_owner, meta_addr);
        assert!(is_asset_deposit_blocked(meta_addr));
        assert!(is_asset_withdrawal_blocked(meta_addr));

        let remote_recipient = dummy_remote_recipient();

        deposit_native_tokens(
            user,
            meta_addr,
            remote_recipient,
            10,
        );
    }

    /// block_asset blocks native withdrawals.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    #[expected_failure(abort_code = E_ASSET_BLOCKED, location = bridge::bridge)]
    fun test_block_asset_blocks_native_withdrawal(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (_meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        whitelist_native_token(bridge_owner, meta_addr);

        // Seed vault
        deposit_native_tokens(
            user,
            meta_addr,
            dummy_remote_recipient(),
            40,
        );

        block_asset(bridge_owner, meta_addr);
        assert!(is_asset_withdrawal_blocked(meta_addr));

        let src_nonce = 777;
        withdraw_tokens(
            bridge_owner,
            meta_addr,
            user2_addr,
            10,
            src_nonce
        );
    }

    /// block_asset blocks wrapped deposits.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_ASSET_BLOCKED, location = bridge::bridge)]
    fun test_block_asset_blocks_wrapped_deposits(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets,
        NativeWhitelist,
    {
        let (_wrapped_meta, wrapped_meta_addr, _user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        block_asset(bridge_owner, wrapped_meta_addr);
        assert!(is_asset_deposit_blocked(wrapped_meta_addr));

        deposit_wrapped_tokens(
            user,
            wrapped_meta_addr,
            dummy_remote_recipient(),
            10,
        );
    }

    /// block_asset blocks wrapped withdrawals.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_ASSET_BLOCKED, location = bridge::bridge)]
    fun test_block_asset_blocks_wrapped_withdrawal(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets,
        NativeWhitelist,
        NativeVault,
        NativeVaults,
    {
        let (_wrapped_meta, wrapped_meta_addr, user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        block_asset(bridge_owner, wrapped_meta_addr);
        assert!(is_asset_withdrawal_blocked(wrapped_meta_addr));

        let src_nonce = 777;

        withdraw_tokens(
            bridge_owner,
            wrapped_meta_addr,
            user_addr,
            10,
            src_nonce
        );
    }

    /// unblock_asset on a native asset restores both deposits and withdrawals.
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE, user2 = @0xBEEF)]
    fun test_unblock_native_asset_unblock_deposits_and_withdrawals(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
        user2: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets
    {
        let (meta, meta_addr, user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        whitelist_native_token(bridge_owner, meta_addr);

        // Seed vault
        deposit_native_tokens(
            user,
            meta_addr,
            dummy_remote_recipient(),
            40,
        );

        block_asset(bridge_owner, meta_addr);
        assert!(is_asset_deposit_blocked(meta_addr));
        assert!(is_asset_withdrawal_blocked(meta_addr));

        unblock_asset(bridge_owner, meta_addr);
        assert!(!is_asset_deposit_blocked(meta_addr));
        assert!(!is_asset_withdrawal_blocked(meta_addr));

        // Deposit and withdraw should now succeed.
        deposit_native_tokens(
            user,
            meta_addr,
            dummy_remote_recipient(),
            10,
        );

        let src_nonce = 777;
        withdraw_tokens(
            bridge_owner,
            meta_addr,
            user2_addr,
            20,
            src_nonce
        );

        let vaults = borrow_global<NativeVaults>(@bridge);
        let vault_addr = *table::borrow(&vaults.vaults, meta_addr);
        let vault_store = object::address_to_object<FungibleStore>(vault_addr);
        let vault_balance = fa::balance(vault_store);

        let user_balance = primary_fungible_store::balance(user_addr, meta);
        let user2_balance = primary_fungible_store::balance(user2_addr, meta);

        assert!(vault_balance + user_balance + user2_balance == 100);
    }

    /// unblock_asset on a wrapped asset restores both deposits and withdrawals.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    fun test_unblock_wrapped_asset_unblock_deposits_and_withdrawals(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets,
        NativeWhitelist,
        NativeVault,
        NativeVaults,
    {
        let (wrapped_meta, wrapped_meta_addr, user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        block_asset(bridge_owner, wrapped_meta_addr);
        assert!(is_asset_deposit_blocked(wrapped_meta_addr));
        assert!(is_asset_withdrawal_blocked(wrapped_meta_addr));

        unblock_asset(bridge_owner, wrapped_meta_addr);
        assert!(!is_asset_deposit_blocked(wrapped_meta_addr));
        assert!(!is_asset_withdrawal_blocked(wrapped_meta_addr));

        // Deposit should succeed
        deposit_wrapped_tokens(
            user,
            wrapped_meta_addr,
            dummy_remote_recipient(),
            40,
        );

        let src_nonce = 1;

        // Withdraw should also succeed
        withdraw_tokens(
            bridge_owner,
            wrapped_meta_addr,
            user_addr,
            10,
            src_nonce
        );

        // Sanity: we mainly care that no aborts happen.
        assert!(next_deposit_nonce() == 1);
        assert!(remote_nonce_processed(src_nonce));
    }

    /// unblock_asset_deposits does NOT unblock withdrawals for wrapped assets.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_ASSET_BLOCKED, location = bridge::bridge)]
    fun test_unblock_wrapped_asset_deposits_do_not_unblock_withdrawals(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets,
        NativeWhitelist,
        NativeVault,
        NativeVaults,
    {
        let (_wrapped_meta, wrapped_meta_addr, user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        block_asset(bridge_owner, wrapped_meta_addr);
        unblock_asset_deposits(bridge_owner, wrapped_meta_addr);

        let src_nonce = 777;

        // Withdraw should still be blocked
        withdraw_tokens(
            bridge_owner,
            wrapped_meta_addr,
            user_addr,
            10,
            src_nonce
        );
    }

    /// unblock_asset_withdrawals does NOT unblock deposits for wrapped assets.
    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_ASSET_BLOCKED, location = bridge::bridge)]
    fun test_unblock_wrapped_asset_withdrawals_do_not_unblock_deposits(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        WrappedAssetsRegistry,
        Requests,
        BridgeEvents,
        Multisig,
        BlockedAssets,
        NativeWhitelist
    {
        let (_wrapped_meta, wrapped_meta_addr, _user_addr) =
            setup_bridge_and_wrapped_fa_for_test(bridge_owner, user);

        block_asset(bridge_owner, wrapped_meta_addr);
        unblock_asset_withdrawals(bridge_owner, wrapped_meta_addr);

        // Deposits should still be blocked
        deposit_wrapped_tokens(
            user,
            wrapped_meta_addr,
            dummy_remote_recipient(),
            10,
        );
    }

    /// rotate_multisig: new multisig can call multisig-only functions (create_wrapped_asset).
    #[test(bridge_owner = @bridge, old_ms = @0xAAA1, new_ms = @0xAAA2)]
    fun test_rotate_multisig_allows_new_multisig(
        bridge_owner: &signer,
        old_ms: &signer,
        new_ms: &signer,
    ) acquires WrappedAssetsRegistry, Multisig {
        // Initialize with old_ms as initial multisig
        initialize(bridge_owner, signer::address_of(old_ms));

        // Rotate to new_ms
        rotate_multisig(bridge_owner, signer::address_of(new_ms));

        // New multisig should now be allowed to create a wrapped asset
        let origin_token = vector[
            0x01u8, 0x02u8, 0x03u8, 0x04u8, 0x05u8,
            0x06u8, 0x07u8, 0x08u8, 0x09u8, 0x0Au8,
            0x0Bu8, 0x0Cu8, 0x0Du8, 0x0Eu8, 0x0Fu8,
            0x10u8, 0x11u8, 0x12u8, 0x13u8, 0x14u8
        ];

        let _wrapped_meta = create_wrapped_asset(
            new_ms,
            &origin_token,
            &b"RotTest",
            &b"RT",
            6,
            &b"",
            &b"",
        );
        // If we reach here, assert_multisig accepted new_ms as Multisig.multisig
    }

        /// rotate_multisig: old multisig can no longer call multisig-only functions.
    #[test(bridge_owner = @bridge, old_ms = @0xAAA1, new_ms = @0xAAA2)]
    #[expected_failure(abort_code = E_NOT_ADMIN, location = bridge::bridge)]
    fun test_rotate_multisig_old_multisig_fails(
        bridge_owner: &signer,
        old_ms: &signer,
        new_ms: &signer,
    ) acquires WrappedAssetsRegistry, Multisig {
        // Initialize with old_ms as initial multisig
        initialize(bridge_owner, signer::address_of(old_ms));

        // Rotate to new_ms
        rotate_multisig(bridge_owner, signer::address_of(new_ms));

        // Old multisig should now fail on multisig-only ops
        let origin_token = vector[
            0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8, 0xEEu8,
            0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8,
            0x66u8, 0x77u8, 0x88u8, 0x99u8, 0xAAu8,
            0xBBu8, 0xCCu8, 0xDDu8, 0xEEu8, 0xFFu8
        ];

        // This must abort in assert_multisig with E_NOT_ADMIN
        let _wrapped_meta = create_wrapped_asset(
            old_ms,
            &origin_token,
            &b"OldFail",
            &b"OF",
            6,
            &b"",
            &b"",
        );
    }
}