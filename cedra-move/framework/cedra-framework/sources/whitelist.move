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

    /// Stores all assets that allowed in transaction commission
    struct FungibleAssetRegistry has key {
        assets: vector<FungibleAssetStruct>
    }

    /// Stores Asset values.
    /// `module_name` is retained so existing on-chain registry data can still be loaded;
    /// identity is `(addr, symbol)` only.
    struct FungibleAssetStruct has copy, drop, store {
        addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
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

    /// View projection of a whitelist entry. Omits the legacy `module_name` storage field.
    struct WhitelistAsset has copy, drop {
        addr: address,
        symbol: vector<u8>
    }


    public entry fun init_registry(admin: &signer) {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        assert_registry_absent(@admin);

        let assets = vector::empty<FungibleAssetStruct>();

        // Add default asset: 0x1 CedraCoin. module_name kept for on-chain layout compatibility.
        vector::push_back(
            &mut assets,
            FungibleAssetStruct {
                addr: @0x1,
                module_name: b"cedra_coin",
                symbol: b"CedraCoin"
            }
        );

        move_to(
            admin,
            FungibleAssetRegistry {
                assets
            }
        );

        emit(
            AssetAddedEvent {
                addr: @0x1,
                module_name: b"cedra_coin",
                symbol: b"CedraCoin"
            }
        );
    }

    // Add asset into FungibleAssetRegistry. Can be used only by admin.
    // `module_name` is accepted for governance/ABI compatibility with already-published calls
    // and is stored so existing on-chain layout keeps loading; identity is `(addr, symbol)`.
    public entry fun add_asset(
        admin: &signer,
        asset_addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    ) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);

        assert!(has_registry(@admin), ENO_REGISTRY);
        assert!(
            admin_address == @admin || admin_address == @0x1,
            EUNAUTHORIZED
        );

        assert!(
            stablecoin::asset_deployed(asset_addr, symbol),
            EASSET_NOT_FOUND
        );

        assert!(
            !asset_exists(asset_addr, vector::empty(), symbol),
            EASSET_EXISTS
        );

        let registry = borrow_global_mut<FungibleAssetRegistry>(@admin);

        vector::push_back(
            &mut registry.assets,
            FungibleAssetStruct { addr: asset_addr, module_name, symbol }
        );

         emit(
            AssetAddedEvent {
                addr: asset_addr,
                module_name,
                symbol
            }
        );
    }

    // Remove asset from FungibleAssetRegistry. Can be used only by admin.
    // `module_name` is accepted for governance/ABI compatibility and ignored for lookup.
    public entry fun remove_asset(
        admin: &signer,
        asset_addr: address,
        _module_name: vector<u8>,
        symbol: vector<u8>
    ) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        let registry = borrow_global_mut<FungibleAssetRegistry>(admin_address);

        let (exist, index) = find_asset_index(&registry.assets, asset_addr, symbol);
        if (exist) {
            let removed = vector::remove(&mut registry.assets, index);

           emit(
                AssetRemovedEvent {
                    addr: removed.addr,
                    module_name: removed.module_name,
                    symbol: removed.symbol
                }
            );
        } else {
            abort EASSET_NOT_FOUND
        }
    }

    // Add CedraCoin into FungibleAssetRegistry. Can be used only by admin
    public entry fun add_cedra_coin(
        admin: &signer,
    ) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);

        assert!(has_registry(@admin), ENO_REGISTRY);
        assert!(
            admin_address == @admin || admin_address == @0x1,
            EUNAUTHORIZED
        );

        assert!(
            !asset_exists(@0x1, vector::empty(), b"CedraCoin"),
            EASSET_EXISTS
        );

        let registry = borrow_global_mut<FungibleAssetRegistry>(@admin);

        vector::push_back(
            &mut registry.assets,
            FungibleAssetStruct { addr: @0x1, module_name: b"cedra_coin", symbol: b"CedraCoin"}
        );

        emit(
            AssetAddedEvent {
                addr: @0x1,
                module_name: b"cedra_coin",
                symbol: b"CedraCoin"
            }
        );
    }


    public(friend) fun asset_exists(
        asset_addr: address, _module_name: vector<u8>, symbol: vector<u8>
    ): bool acquires FungibleAssetRegistry {
        let registry = borrow_global<FungibleAssetRegistry>(@admin);
        let (exist, _) = find_asset_index(&registry.assets, asset_addr, symbol);
        exist
    }

    fun find_asset_index(
        assets: &vector<FungibleAssetStruct>, asset_addr: address, symbol: vector<u8>
    ): (bool, u64) {
        let i = 0;
        let n = vector::length(assets);
        while (i < n) {
            let asset = vector::borrow(assets, i);
            if (asset.addr == asset_addr && asset.symbol == symbol) {
                return (true, i);
            };
            i = i + 1;
        };
        (false, 0)
    }

    public(friend) fun has_registry(addr: address): bool {
        exists<FungibleAssetRegistry>(addr)
    }

    fun assert_registry_absent(admin_address: address) {
        assert!(!exists<FungibleAssetRegistry>(admin_address), EALREADY_INITIALIZED);
    }

    #[view]
    public fun get_asset_list(
        admin: address
    ): vector<FungibleAssetStruct> acquires FungibleAssetRegistry {
        borrow_global<FungibleAssetRegistry>(admin).assets
    }

    #[view]
    public fun get_asset_list_v2(
        admin: address
    ): vector<WhitelistAsset> acquires FungibleAssetRegistry {
        let stored = &borrow_global<FungibleAssetRegistry>(admin).assets;
        let out = vector::empty<WhitelistAsset>();
        let i = 0;
        let n = vector::length(stored);
        while (i < n) {
            let asset = vector::borrow(stored, i);
            vector::push_back(
                &mut out,
                WhitelistAsset { addr: asset.addr, symbol: asset.symbol }
            );
            i = i + 1;
        };
        out
    }
}
