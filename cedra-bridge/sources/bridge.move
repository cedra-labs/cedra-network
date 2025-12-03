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

    /* ===================== Admin (multisig) ===================== */

    struct Admin has key {
        multisig: address,
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

    /* ===================== Global config ===================== */

    struct Config has key { paused: bool }

    /* ===================== FA caps and registry (wrapped only) ===================== */

    struct FACaps has store {
        mint: MintRef,
        burn: BurnRef,
        transfer: TransferRef,
    }

    struct FARegistry has key {
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
        used_nonce: Table<u64, bool>, // shared for all directions, like usedNonces on ETH bridge
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

    inline fun assert_not_paused() acquires Config {
        assert!(!borrow_global<Config>(@bridge).paused, E_PAUSED);
    }

    inline fun assert_owner(s: &signer) {
        // Only module owner (@bridge) can do certain bootstrap actions.
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

    inline fun get_requests(): &mut Requests {
        borrow_global_mut<Requests>(@bridge)
    }

    inline fun get_whitelist(): &mut NativeWhitelist {
        borrow_global_mut<NativeWhitelist>(@bridge)
    }

    inline fun get_vaults(): &mut NativeVaults {
        borrow_global_mut<NativeVaults>(@bridge)
    }

    inline fun mark_nonce_used(reqs: &mut Requests, nonce: u64) {
        assert!(!table::contains(&reqs.used_nonce, nonce), E_NONCE_USED);
        table::add(&mut reqs.used_nonce, nonce, true);
    }

    inline fun get_caps_or_abort(meta_addr: address): &mut FACaps acquires FARegistry {
        let reg = borrow_global_mut<FARegistry>(@bridge);
        assert!(table::contains(&reg.caps_by_meta, meta_addr), E_ASSET_UNKNOWN);
        table::borrow_mut(&mut reg.caps_by_meta, meta_addr)
    }

    inline fun get_wrapped_meta_for_origin_or_abort(origin_token: &vector<u8>): Object<Metadata> acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
        assert!(table::contains(&reg.origin_to_wrapped, *origin_token), E_ASSET_UNKNOWN);
        *table::borrow(&reg.origin_to_wrapped, *origin_token)
    }

    /* ===================== Init & admin ===================== */

    public entry fun initialize(bridge_owner: &signer) {
        assert_framework(bridge_owner);
        assert!(!exists<Config>(@bridge), E_ALREADY_INITIALIZED);

        move_to(bridge_owner, Config { paused: false });

        move_to(bridge_owner, FARegistry {
            origin_to_wrapped: table::new<vector<u8>, Object<Metadata>>(),
            wrapped_to_origin: table::new<address, vector<u8>>(),
            caps_by_meta:      table::new<address, FACaps>(),
            origin_tokens:     vector::empty<vector<u8>>(),
            wrapped_meta_addrs: vector::empty<address>(),
        });

        move_to(bridge_owner, NativeWhitelist {
            native_assets: table::new<address, bool>(),
        });

        move_to(bridge_owner, NativeVaults {
            vaults: table::new<address, address>(),
        });

        move_to(bridge_owner, Requests {
            used_nonce: table::new<u64, bool>(),
        });

        move_to(bridge_owner, BridgeEvents {
            deposit:  account::new_event_handle<Deposit>(bridge_owner),
            withdraw: account::new_event_handle<Withdraw>(bridge_owner),
        });

        // Initial multisig = module owner; you can rotate later.
        move_to(bridge_owner, Admin { multisig: signer::address_of(bridge_owner) });
    }

    /// Set initial multisig explicitly (only framework, not end-user).
    public entry fun set_initial_multisig(bridge_owner: &signer, addr: address) acquires Admin {
        assert_owner(bridge_owner);
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
    ): Object<Metadata> acquires FARegistry, Admin {
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
        multisig: &signer,
        native_meta_addr: address,
    ) acquires NativeWhitelist, FARegistry, Admin {
        assert_multisig(multisig);

        // Must NOT be a wrapped asset: wrapped assets have caps_by_meta entries.
        let reg = borrow_global<FARegistry>(@bridge);
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
    }

    /// Multisig-only: remove from native whitelist.
    public entry fun unwhitelist_native_token(
        multisig: &signer,
        native_meta_addr: address,
    ) acquires NativeWhitelist, Admin {
        assert_multisig(multisig);

        let wl = get_whitelist();
        assert!(
            table::contains(&wl.native_assets, native_meta_addr),
            E_ASSET_UNKNOWN
        );
        let _ = table::remove(&mut wl.native_assets, native_meta_addr);
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
        nonce: u64,
    ) acquires FARegistry, NativeWhitelist, NativeVaults, Requests, BridgeEvents, Config {
        assert_not_paused();
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&remote_recipient);

        // Must be whitelisted & NOT wrapped
        let wl = borrow_global<NativeWhitelist>(@bridge);
        assert!(table::contains(&wl.native_assets, native_meta_addr), E_ASSET_MISMATCH);

        let reg = borrow_global<FARegistry>(@bridge);
        assert!(
            !table::contains(&reg.caps_by_meta, native_meta_addr),
            E_ASSET_MISMATCH
        );

        let reqs = get_requests();
        mark_nonce_used(reqs, nonce);

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
        nonce: u64,
    ) acquires FARegistry, Requests, BridgeEvents, Config {
        assert_not_paused();
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&remote_recipient);

        // Must be a known wrapped asset (has caps)
        {
            let reg = borrow_global<FARegistry>(@bridge);
            assert!(
                table::contains(&reg.caps_by_meta, wrapped_meta_addr),
                E_ASSET_MISMATCH
            );
        };

        let reqs = get_requests();
        mark_nonce_used(reqs, nonce);

        let meta: Object<Metadata> = object::address_to_object<Metadata>(wrapped_meta_addr);
        let withdrawn: FungibleAsset = primary_fungible_store::withdraw<Metadata>(user, meta, amount);

        let caps = get_caps_or_abort(wrapped_meta_addr);
        fa::burn(&caps.burn, withdrawn);

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
        nonce: u64,
    ) acquires
        FARegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Config,
        Admin {
        assert_not_paused();
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);

        let reqs = get_requests();
        mark_nonce_used(reqs, nonce);

        let reg = borrow_global<FARegistry>(@bridge);
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
            ev::emit(Withdraw { asset: meta_addr, to, amount, nonce });
        } else {
            ev::emit_event(&mut evs.withdraw, Withdraw { asset: meta_addr, to, amount, nonce });
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
        nonce: u64,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
        icon_uri: vector<u8>,
        project_uri: vector<u8>,
    ) acquires FARegistry, Requests, BridgeEvents, Config, Admin {
        assert_not_paused();
        assert_multisig(multisig);
        assert!(amount > 0, E_ZERO_AMOUNT);
        assert_20_bytes(&origin_token);

        let reqs = get_requests();
        mark_nonce_used(reqs, nonce);

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
        let caps = get_caps_or_abort(wrapped_meta_addr);
        let minted: FungibleAsset = fa::mint(&caps.mint, amount);
        primary_fungible_store::deposit(to, minted);

        let evs = borrow_global_mut<BridgeEvents>(@bridge);
        if (features::module_event_migration_enabled()) {
            ev::emit(Withdraw { asset: wrapped_meta_addr, to, amount, nonce });
        } else {
            ev::emit_event(
                &mut evs.withdraw,
                Withdraw { asset: wrapped_meta_addr, to, amount, nonce },
            );
        };
    }

    /* ===================== Views / helpers ===================== */

    #[view]
    public fun admin_multisig(): address acquires Admin {
        borrow_global<Admin>(@bridge).multisig
    }

    #[view]
    public fun nonce_used(n: u64): bool acquires Requests {
        table::contains(&borrow_global<Requests>(@bridge).used_nonce, n)
    }

    /// Check if a Cedra asset (by Metadata address) is wrapped.
    /// Logic: wrapped assets are exactly those with caps in `caps_by_meta`.
    #[view]
    public fun is_wrapped_asset(meta_addr: address): bool acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
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
    public fun wrapped_meta_of_origin(origin_token: vector<u8>): address acquires FARegistry {
        let meta = get_wrapped_meta_for_origin_or_abort(&origin_token);
        object::object_address(&meta)
    }

    #[view]
    public fun origin_of_wrapped(meta_addr: address): vector<u8> acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
        assert!(table::contains(&reg.wrapped_to_origin, meta_addr), E_ASSET_UNKNOWN);
        *table::borrow(&reg.wrapped_to_origin, meta_addr)
    }

    #[view]
    public fun all_wrapped_meta_addrs(): vector<address> acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
        reg.wrapped_meta_addrs
    }

    #[view]
    public fun all_origin_tokens(): vector<vector<u8>> acquires FARegistry {
        let reg = borrow_global<FARegistry>(@bridge);
        reg.origin_tokens
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
        initialize(bridge_owner);

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
        FARegistry,
        NativeWhitelist,
        NativeVaults,
        NativeVault,
        Requests,
        BridgeEvents,
        Admin
    {
        let (meta, meta_addr, user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        let user2_addr = signer::address_of(user2);

        // bridge_owner is also multisig in tests
        assert!(admin_multisig() == signer::address_of(bridge_owner));

        // Whitelist native token
        whitelist_native_token(bridge_owner, meta_addr);
        assert!(is_native_asset(meta_addr));

        let before_user = primary_fungible_store::balance(user_addr, meta);

        // Dummy 20-byte Ethereum recipient
        let remote_recipient = vector[
            1u8, 2u8, 3u8, 4u8, 5u8,
            6u8, 7u8, 8u8, 9u8, 10u8,
            11u8, 12u8, 13u8, 14u8, 15u8,
            16u8, 17u8, 18u8, 19u8, 20u8
        ];

        // deposit_native_tokens: lock 200 into vault
        deposit_native_tokens(
            user,
            meta_addr,
            remote_recipient,
            60,
            1,
        );

        // Nonce used
        assert!(nonce_used(1));

        // User lost 200
        let after_user = primary_fungible_store::balance(user_addr, meta);
        assert!(after_user == before_user - 60);

        // Vault exists and holds 60
        let vaults = borrow_global<NativeVaults>(@bridge);
        let vault_addr = *table::borrow(&vaults.vaults, meta_addr);
        let vault_store = object::address_to_object<FungibleStore>(vault_addr);
        assert!(fa::balance(vault_store) == 60);

        // withdraw_tokens (native path): unlock 120 to user2
        withdraw_tokens(
            bridge_owner,
            meta_addr,
            user2_addr,
            25,
            2,
        );

        assert!(nonce_used(2));

        let vault_after = fa::balance(vault_store);
        let user2_balance = primary_fungible_store::balance(user2_addr, meta);

        // 60 - 25 = 35 left in vault
        assert!(vault_after == 35);
        assert!(user2_balance == 25);
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
        FARegistry,
        Requests,
        BridgeEvents,
        Admin,
        NativeWhitelist,
    {
        initialize(bridge_owner);
        let user_addr = signer::address_of(user);

        // Example origin ERC20 (20 bytes)
        let origin_token = vector[
            0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8, 0xEEu8,
            0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8,
            0x66u8, 0x77u8, 0x88u8, 0x99u8, 0xAAu8,
            0xBBu8, 0xCCu8, 0xDDu8, 0xEEu8, 0xFFu8
        ];

        // First withdraw: should create wrapped FA and mint 500
        withdraw_auto_create_wrapped(
            bridge_owner,
            copy origin_token,
            user_addr,
            500,
            10,
            b"CedraWrapped",
            b"cTOK",
            6,
            b"",
            b"",
        );

        assert!(nonce_used(10));

        let reg = borrow_global<FARegistry>(@bridge);
        assert!(table::contains(&reg.origin_to_wrapped, origin_token));
        let wrapped_meta_1 = *table::borrow(&reg.origin_to_wrapped, origin_token);
        let wrapped_meta_addr_1 = object::object_address(&wrapped_meta_1);

        assert!(is_wrapped_asset(wrapped_meta_addr_1));
        assert!(!is_native_asset(wrapped_meta_addr_1));

        // User should have 500 wrapped
        assert!(primary_fungible_store::balance(user_addr, wrapped_meta_1) == 500);

        // ------------------------------------------------------
        // NEW: user transfers part of wrapped tokens to another address
        // ------------------------------------------------------
        let recipient_addr = @0xBEEF;

        let before_user     = primary_fungible_store::balance(user_addr, wrapped_meta_1);
        let before_recipient = primary_fungible_store::balance(recipient_addr, wrapped_meta_1);

        let transfer_amount = 200;

        // This uses the standard FA primary-store transfer API.
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

        // ------------------------------------------------------
        // Second withdraw: should reuse same wrapped asset and mint +100
        // ------------------------------------------------------
        withdraw_auto_create_wrapped(
            bridge_owner,
            copy origin_token,
            user_addr,
            100,
            11,
            // Name/symbol ignored when already exists
            b"IGNORED",
            b"IGN",
            6,
            b"",
            b"",
        );

        assert!(nonce_used(11));

        let reg2 = borrow_global<FARegistry>(@bridge);
        let wrapped_meta_2 = *table::borrow(&reg2.origin_to_wrapped, origin_token);
        let wrapped_meta_addr_2 = object::object_address(&wrapped_meta_2);

        // Must point to same Metadata object
        assert!(wrapped_meta_addr_2 == wrapped_meta_addr_1);

        // Total supply check is a bit trickier now because some tokens are on recipient,
        // but we can still sanity-check user's new balance:
        // original 500 - 200 transfer + 100 second withdraw = 400
        assert!(primary_fungible_store::balance(user_addr, wrapped_meta_2) == 400);

        // View helper for origin > wrapped
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
        FARegistry,
        NativeWhitelist,
        Requests,
        BridgeEvents,
        Admin
    {
        initialize(bridge_owner);
        let user_addr = signer::address_of(user);

        // Create a wrapped asset for some origin ERC20
        let origin_token = vector[
            1u8, 1u8, 1u8, 1u8, 1u8,
            2u8, 2u8, 2u8, 2u8, 2u8,
            3u8, 3u8, 3u8, 3u8, 3u8,
            4u8, 4u8, 4u8, 4u8, 4u8
        ];

        let wrapped_meta = create_wrapped_asset(
            bridge_owner,
            &origin_token,
            &b"CedraW",
            &b"cW",
            6,
            &b"",
            &b"",
        );
        let wrapped_meta_addr = object::object_address(&wrapped_meta);

        // Mint 500 wrapped to user via bridge mint caps
        let caps = get_caps_or_abort(wrapped_meta_addr);
        let minted: FungibleAsset = fa::mint(&caps.mint, 500);
        primary_fungible_store::deposit(user_addr, minted);

        assert!(primary_fungible_store::balance(user_addr, wrapped_meta) == 500);
        assert!(is_wrapped_asset(wrapped_meta_addr));
        assert!(!is_native_asset(wrapped_meta_addr));

        let remote_recipient = vector[
            9u8, 9u8, 9u8, 9u8, 9u8,
            8u8, 8u8, 8u8, 8u8, 8u8,
            7u8, 7u8, 7u8, 7u8, 7u8,
            6u8, 6u8, 6u8, 6u8, 6u8
        ];

        deposit_wrapped_tokens(
            user,
            wrapped_meta_addr,
            remote_recipient,
            500,
            21,
        );

        assert!(nonce_used(21));
        // Entire balance burned
        assert!(primary_fungible_store::balance(user_addr, wrapped_meta) == 0);
    }

    /// Pause / unpause:
    /// - pause blocks deposit_native_tokens and withdraw_auto_create_wrapped
    /// - unpause allows them again
    #[test(bridge_owner = @bridge, asset_admin = @0xAAAAA, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_pause_blocks_deposit_native(
        bridge_owner: &signer,
        asset_admin: &signer,
        user: &signer,
    ) acquires
        Config,
        FARegistry,
        NativeWhitelist,
        NativeVaults,
        Requests,
        BridgeEvents,
        Admin
    {
        let (meta, meta_addr, _user_addr) =
            setup_bridge_and_native_fa_for_test(bridge_owner, asset_admin, user);
        whitelist_native_token(bridge_owner, meta_addr);

        pause(bridge_owner);

        let remote_recipient = vector[
            0u8, 0u8, 0u8, 0u8, 0u8,
            1u8, 1u8, 1u8, 1u8, 1u8,
            2u8, 2u8, 2u8, 2u8, 2u8,
            3u8, 3u8, 3u8, 3u8, 3u8
        ];

        // Must abort with E_PAUSED
        deposit_native_tokens(
            user,
            meta_addr,
            remote_recipient,
            10,
            100,
        );
    }

    #[test(bridge_owner = @bridge, user = @0xCAFE)]
    #[expected_failure(abort_code = E_PAUSED, location = bridge::bridge)]
    fun test_pause_blocks_withdraw_auto_create_wrapped(
        bridge_owner: &signer,
        user: &signer,
    ) acquires
        Config,
        FARegistry,
        Requests,
        BridgeEvents,
        Admin
    {
        initialize(bridge_owner);
        pause(bridge_owner);

        let user_addr = signer::address_of(user);
        let origin_token = vector[
            0x10u8, 0x11u8, 0x12u8, 0x13u8, 0x14u8,
            0x15u8, 0x16u8, 0x17u8, 0x18u8, 0x19u8,
            0x20u8, 0x21u8, 0x22u8, 0x23u8, 0x24u8,
            0x25u8, 0x26u8, 0x27u8, 0x28u8, 0x29u8
        ];

        // Must abort with E_PAUSED
        withdraw_auto_create_wrapped(
            bridge_owner,
            origin_token,
            user_addr,
            100,
            101,
            b"Paused",
            b"P",
            6,
            b"",
            b"",
        );
    }
}