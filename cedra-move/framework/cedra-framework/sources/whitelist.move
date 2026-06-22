// This module provides an interface to store, add and remove assets to whitelist registry.
module cedra_framework::whitelist {
    use std::vector;
    use std::signer;
    use cedra_framework::event::emit;

    use cedra_framework::stablecoin;

    friend cedra_framework::transaction_fee;

    /// Caller is not authorized to make this call
    const EUNAUTHORIZED: u64 = 1;
    // Fungible Asset not exist in FungibleAssetRegistry
    const EASSET_NOT_FOUND: u64 = 2;
    // FungibleAssetRegistry already initialized
    const EALREADY_INITIALIZED: u64 = 3;
    const ENO_REGISTRY: u64 = 4;
    const EASSET_EXISTS: u64 = 5;
    const ENO_LEGACY_REGISTRY: u64 = 6;

    const CEDRA_COIN_SYMBOL: vector<u8> = b"CedraCoin";
    const STABLECOIN_MODULE: vector<u8> = b"stablecoin";
    const CEDRA_COIN_MODULE: vector<u8> = b"cedra_coin";

    // -------------------------------------------------------------------------
    // Legacy on-chain layout (addr + module_name + symbol).
    // Preserved for backward-compatible module upgrades and API surface.
    // -------------------------------------------------------------------------
    struct FungibleAssetStruct has copy, drop, store {
        addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    }

    struct FungibleAssetRegistry has key {
        assets: vector<FungibleAssetStruct>
    }

    // -------------------------------------------------------------------------
    // Canonical storage (addr + symbol only).
    // -------------------------------------------------------------------------
    struct WhitelistAsset has copy, drop, store {
        addr: address,
        symbol: vector<u8>
    }

    struct WhitelistRegistry has key {
        assets: vector<WhitelistAsset>
    }

    #[event]
    struct AssetAddedEvent has copy, drop, store {
        addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    }

    #[event]
    struct AssetRemovedEvent has copy, drop, store {
        addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    }

    public entry fun init_registry(admin: &signer) {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);
        assert_no_registry(@admin);

        let assets = vector::empty<WhitelistAsset>();
        vector::push_back(&mut assets, cedra_coin_asset());

        move_to(
            admin,
            WhitelistRegistry {
                assets
            }
        );

        emit(
            AssetAddedEvent {
                addr: @0x1,
                module_name: CEDRA_COIN_MODULE,
                symbol: CEDRA_COIN_SYMBOL
            }
        );
    }

    /// Migrates the legacy registry into `WhitelistRegistry` and removes the old resource.
    public entry fun migrate_registry(admin: &signer) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);
        assert!(exists<FungibleAssetRegistry>(@admin), ENO_LEGACY_REGISTRY);
        assert!(!exists<WhitelistRegistry>(@admin), EALREADY_INITIALIZED);
        migrate_registry_internal(admin);
    }

    /// Destroys both legacy and canonical registries.
    public entry fun destroy_registry(admin: &signer) acquires FungibleAssetRegistry, WhitelistRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);
        assert!(has_registry(@admin), ENO_REGISTRY);

        if (exists<WhitelistRegistry>(@admin)) {
            let WhitelistRegistry { assets: _ } = move_from<WhitelistRegistry>(@admin);
        };
        if (exists<FungibleAssetRegistry>(@admin)) {
            let FungibleAssetRegistry { assets: _ } = move_from<FungibleAssetRegistry>(@admin);
        };
    }

    // Backward-compatible entry: `module_name` is accepted for API compatibility but
    // matching uses addr + symbol only.
    public entry fun add_asset(
        admin: &signer,
        asset_addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    ) acquires FungibleAssetRegistry, WhitelistRegistry {
        let admin_address = signer::address_of(admin);

        ensure_canonical_registry(admin);
        assert!(
            admin_address == @admin || admin_address == @0x1,
            EUNAUTHORIZED
        );

        assert!(
            stablecoin::asset_deployed(asset_addr, symbol),
            EASSET_NOT_FOUND
        );

        assert!(
            !asset_exists(asset_addr, symbol),
            EASSET_EXISTS
        );

        let registry = borrow_global_mut<WhitelistRegistry>(@admin);

        vector::push_back(
            &mut registry.assets,
            WhitelistAsset { addr: asset_addr, symbol }
        );

        emit(
            AssetAddedEvent {
                addr: asset_addr,
                module_name,
                symbol
            }
        );
    }

    public entry fun remove_asset(
        admin: &signer,
        asset_addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    ) acquires FungibleAssetRegistry, WhitelistRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        ensure_canonical_registry(admin);

        let registry = borrow_global_mut<WhitelistRegistry>(admin_address);

        let (exist, index) = vector::index_of(
            &registry.assets,
            &WhitelistAsset { addr: asset_addr, symbol }
        );
        if (exist) {
            vector::remove(&mut registry.assets, index);

            emit(
                AssetRemovedEvent {
                    addr: asset_addr,
                    module_name,
                    symbol
                }
            );
        } else {
            abort EASSET_NOT_FOUND
        }
    }

    public entry fun add_cedra_coin(
        admin: &signer,
    ) acquires FungibleAssetRegistry, WhitelistRegistry {
        let admin_address = signer::address_of(admin);

        ensure_canonical_registry(admin);
        assert!(
            admin_address == @admin || admin_address == @0x1,
            EUNAUTHORIZED
        );

        assert!(
            !asset_exists(@0x1, CEDRA_COIN_SYMBOL),
            EASSET_EXISTS
        );

        let registry = borrow_global_mut<WhitelistRegistry>(@admin);

        vector::push_back(
            &mut registry.assets,
            cedra_coin_asset()
        );

        emit(
            AssetAddedEvent {
                addr: @0x1,
                module_name: CEDRA_COIN_MODULE,
                symbol: CEDRA_COIN_SYMBOL
            }
        );
    }

    public(friend) fun asset_exists(
        asset_addr: address, symbol: vector<u8>
    ): bool acquires FungibleAssetRegistry, WhitelistRegistry {
        if (exists<WhitelistRegistry>(@admin)) {
            return asset_exists_canonical(asset_addr, symbol);
        };

        if (exists<FungibleAssetRegistry>(@admin)) {
            return asset_exists_legacy(asset_addr, symbol);
        };

        false
    }

    public(friend) fun has_registry(addr: address): bool {
        exists<WhitelistRegistry>(addr) || exists<FungibleAssetRegistry>(addr)
    }

    #[view]
    public fun is_cedra_coin(addr: address, symbol: vector<u8>): bool {
        addr == @0x1 && symbol == CEDRA_COIN_SYMBOL
    }

    #[view]
    public fun is_migrated(admin: address): bool {
        exists<WhitelistRegistry>(admin)
    }

    #[view]
    public fun get_canonical_asset_list(
        admin: address
    ): vector<WhitelistAsset> acquires FungibleAssetRegistry, WhitelistRegistry {
        if (exists<WhitelistRegistry>(admin)) {
            return borrow_global<WhitelistRegistry>(admin).assets
        };

        if (exists<FungibleAssetRegistry>(admin)) {
            return legacy_assets_to_canonical(
                &borrow_global<FungibleAssetRegistry>(admin).assets
            );
        };

        vector::empty<WhitelistAsset>()
    }

    #[view]
    public fun get_asset_list(
        admin: address
    ): vector<FungibleAssetStruct> acquires FungibleAssetRegistry, WhitelistRegistry {
        if (exists<WhitelistRegistry>(admin)) {
            return canonical_assets_to_legacy(
                &borrow_global<WhitelistRegistry>(admin).assets
            );
        };

        if (exists<FungibleAssetRegistry>(admin)) {
            return borrow_global<FungibleAssetRegistry>(admin).assets;
        };

        vector::empty<FungibleAssetStruct>()
    }

    fun ensure_canonical_registry(admin: &signer) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        if (exists<WhitelistRegistry>(admin_address)) {
            return
        };

        if (exists<FungibleAssetRegistry>(admin_address)) {
            migrate_registry_internal(admin);
            return
        };

        abort ENO_REGISTRY
    }

    fun migrate_registry_internal(admin: &signer) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        let FungibleAssetRegistry { assets: legacy_assets } =
            move_from<FungibleAssetRegistry>(admin_address);

        let canonical_assets = legacy_assets_to_canonical(&legacy_assets);

        move_to(
            admin,
            WhitelistRegistry { assets: canonical_assets }
        );
    }

    fun legacy_assets_to_canonical(
        legacy_assets: &vector<FungibleAssetStruct>
    ): vector<WhitelistAsset> {
        let canonical = vector::empty<WhitelistAsset>();
        let i = 0;
        let n = vector::length(legacy_assets);
        while (i < n) {
            let legacy = vector::borrow(legacy_assets, i);
            let canonical_asset = WhitelistAsset {
                addr: legacy.addr,
                symbol: legacy.symbol
            };
            if (!vector::contains(&canonical, &canonical_asset)) {
                vector::push_back(&mut canonical, canonical_asset);
            };
            i = i + 1;
        };
        canonical
    }

    fun canonical_assets_to_legacy(
        canonical_assets: &vector<WhitelistAsset>
    ): vector<FungibleAssetStruct> {
        let legacy = vector::empty<FungibleAssetStruct>();
        let i = 0;
        let n = vector::length(canonical_assets);
        while (i < n) {
            let asset = vector::borrow(canonical_assets, i);
            vector::push_back(&mut legacy, to_legacy_struct(asset));
            i = i + 1;
        };
        legacy
    }

    fun to_legacy_struct(asset: &WhitelistAsset): FungibleAssetStruct {
        FungibleAssetStruct {
            addr: asset.addr,
            module_name: oracle_module_name(asset.addr, asset.symbol),
            symbol: asset.symbol
        }
    }

    fun oracle_module_name(addr: address, symbol: vector<u8>): vector<u8> {
        if (is_cedra_coin(addr, symbol)) {
            CEDRA_COIN_MODULE
        } else {
            STABLECOIN_MODULE
        }
    }

    fun asset_exists_canonical(
        asset_addr: address, symbol: vector<u8>
    ): bool acquires WhitelistRegistry {
        let registry = borrow_global<WhitelistRegistry>(@admin);
        let i = 0;
        let n = vector::length(&registry.assets);
        while (i < n) {
            let asset = vector::borrow(&registry.assets, i);
            if (asset.addr == asset_addr && asset.symbol == symbol) {
                return true;
            };
            i = i + 1;
        };
        false
    }

    fun asset_exists_legacy(
        asset_addr: address, symbol: vector<u8>
    ): bool acquires FungibleAssetRegistry {
        let registry = borrow_global<FungibleAssetRegistry>(@admin);
        let i = 0;
        let n = vector::length(&registry.assets);
        while (i < n) {
            let asset = vector::borrow(&registry.assets, i);
            if (asset.addr == asset_addr && asset.symbol == symbol) {
                return true;
            };
            i = i + 1;
        };
        false
    }

    fun assert_no_registry(admin_address: address) {
        assert!(
            !exists<WhitelistRegistry>(admin_address)
                && !exists<FungibleAssetRegistry>(admin_address),
            EALREADY_INITIALIZED
        );
    }

    fun cedra_coin_asset(): WhitelistAsset {
        WhitelistAsset { addr: @0x1, symbol: CEDRA_COIN_SYMBOL }
    }
}
