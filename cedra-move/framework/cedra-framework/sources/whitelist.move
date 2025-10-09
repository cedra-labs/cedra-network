// This module provides an interface to store, add and remove assets to whitelist registry.
module cedra_framework::whitelist {
    use std::vector;
    use std::signer;
    use cedra_framework::object::{Self, Object};
    use cedra_framework::fungible_asset::{Self, Metadata};
    use cedra_framework::stablecoin;

    friend cedra_framework::transaction_fee;

    /// Caller is not authorized to make this call
    const EUNAUTHORIZED: u64 = 1;
    // Fungible Asset not exist in FungibleAssetRegistry
    const EASSET_NOT_FOUND: u64 = 2;
    // FungibleAssetRegistry already initialized
    const EALREADY_INITIALIZED: u64 = 3;
    const ENO_REGISTRY: u64 = 4;

    /// Stores all assets that allowed in transaction commission
    struct FungibleAssetRegistry has key {
        assets: vector<FungibleAssetStruct>
    }

    /// Stores Asset values
    struct FungibleAssetStruct has copy, drop, store {
        addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    }

    /// Initialize an empty FungibleAssetRegistry
    public entry fun init_registry(admin: &signer) {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        assert_registry_absent(@admin);

        move_to(
            admin,
            FungibleAssetRegistry {
                assets: vector::empty<FungibleAssetStruct>()
            }
        );
    }

    // Add asset into FungibleAssetRegistry. Can be used only by admin
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

        let registry = borrow_global_mut<FungibleAssetRegistry>(@admin);
        vector::push_back(
            &mut registry.assets,
            FungibleAssetStruct { addr: asset_addr, module_name, symbol }
        );
    }

    // Remove asset from FungibleAssetRegistry. Can be used only by admin
    public entry fun remove_asset(
        admin: &signer,
        asset_addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    ) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        let registry = borrow_global_mut<FungibleAssetRegistry>(admin_address);

        let (exist, index) = vector::index_of(
            &registry.assets,
            &FungibleAssetStruct { addr: asset_addr, module_name, symbol }
        );
        if (exist) {
            vector::remove(&mut registry.assets, index);
        } else {
            abort EASSET_NOT_FOUND
        }
    }

    public(friend) fun asset_exists(
        asset_addr: address, module_name: vector<u8>, symbol: vector<u8>
    ): bool acquires FungibleAssetRegistry {
        let registry = borrow_global<FungibleAssetRegistry>(@admin);

        let i = 0;
        let n = vector::length(&registry.assets);
        while (i < n) {
            let asset = vector::borrow(&registry.assets, i);
            if (asset.addr == asset_addr
                && asset.module_name == module_name
                && asset.symbol == symbol) {
                return true;
            };
            i = i + 1;
        };
        false
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
    /// get_metadata_list returns a list of metadata objects for the existing stablecoins whitelist.
    public fun get_metadata_list(): vector<Object<Metadata>> acquires FungibleAssetRegistry{
        let registry = borrow_global<FungibleAssetRegistry>(@admin);

        let i = 0;
        let n = vector::length(&registry.assets);
        let metadata_list = vector::empty<Object<Metadata>>();

        while (i < n) {
            let asset = vector::borrow(&registry.assets, i);
            let asset_address = object::create_object_address(&asset.addr, asset.symbol);
            let asset_metadata = object::address_to_object(asset_address);
            ///object::address_to_object<Metadata>(asset_address);

            vector::push_back(&mut metadata_list, asset_metadata);

            i = i + 1;
        };

        metadata_list
    }
}

///vector<object::Object<fungible_asset::Metadata>>
///vector<object::Object<fungible_asset::Metadata>>