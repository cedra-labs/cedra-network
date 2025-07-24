//TODO: make spec, test, refactor & optimize all 
module cedra_framework::custom_fungible_asset{
    use cedra_framework::fungible_asset::{Self, MintRef, TransferRef, BurnRef, Metadata};
    use cedra_framework::object::{Self, Object};
    use cedra_framework::primary_fungible_store;
    use std::error;
    use std::signer;
    use std::string::{Self, String};
    use std::option;
    use std::vector;

    /// Errors
    const ENOT_OWNER: u64 = 1;
    const EPAUSED: u64 = 2;
    const EASSET_EXISTS: u64 = 3;
    const EASSET_NOT_FOUND: u64 = 4;
    const EINSUFFICIENT_BALANCE: u64 = 5;

    /// Global registry of created fungible assets
    // TODO: i think is not good option to create wrapper
    // list for FA 
    struct FungibleAssetRegistry has key {
        assets: vector<vector<u8>>,
    }

    /// Per-asset management resources
    struct ManagedFungibleAsset has key {
        mint_ref: MintRef,
        transfer_ref: TransferRef,
        burn_ref: BurnRef,
        symbol: vector<u8>,
    }

    struct AssetState has key {
        paused: bool,
        symbol: vector<u8>,
    }

    /// Initialize the factory
    // Todo: init_module don't work here because we use governance
    // we have 2 options: change governance publish_tx() or call this by hand
    // and create guards to lock after first use
    public entry fun init_registry(admin: &signer) {
        move_to(admin, FungibleAssetRegistry {
            assets: vector::empty(),
        });
    }

    /// Create a new fungible asset with custom parameters
    public entry fun create_fa(
        admin: &signer,
        symbol: vector<u8>,
        name: String,
        decimals: u8,
        icon_url: String,
        project_url: String,
    ) acquires FungibleAssetRegistry {
        // Check if asset already exists
        let registry = borrow_global_mut<FungibleAssetRegistry>(signer::address_of(admin));
        assert!(
            !vector::contains(&registry.assets, &symbol),
            error::already_exists(EASSET_EXISTS)
        );

        let constructor_ref = &object::create_named_object(admin, copy symbol);
        
        // Create the fungible asset
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            constructor_ref,
            option::none(),
            name,
            string::utf8(copy symbol),
            decimals,
            icon_url,
            project_url,
        );

        // Generate control references
        let mint_ref = fungible_asset::generate_mint_ref(constructor_ref);
        let burn_ref = fungible_asset::generate_burn_ref(constructor_ref);
        let transfer_ref = fungible_asset::generate_transfer_ref(constructor_ref);
        let metadata_object_signer = object::generate_signer(constructor_ref);

        // Store control references
        move_to(
            &metadata_object_signer,
            ManagedFungibleAsset {
                mint_ref,
                transfer_ref,
                burn_ref,
                symbol: copy symbol,
            }
        );

        // Initialize state
        move_to(
            &metadata_object_signer,
            AssetState {
                paused: false,
                symbol: copy symbol,
            }
        );

        // Add to registry
        vector::push_back(&mut registry.assets, copy symbol);

    }

    #[view]
    // get address of fungible asset
    public fun fa_address(owner: address, symbol: vector<u8>): address {
        object::create_object_address(&owner, symbol)
    }

    #[view]
    // get metadata of fungible asset
    public fun metadata(owner: address, symbol: vector<u8>): Object<Metadata> {
        object::address_to_object(fa_address(owner, symbol))
    }

    #[view]
    // get list of fungible assets registered in FungibleAssetRegistry
    public fun get_asset_list(admin: address): vector<vector<u8>> acquires FungibleAssetRegistry {
        borrow_global<FungibleAssetRegistry>(admin).assets
    }

    #[view]
    // get balance of fungible asset for account
    public fun get_balance(admin: address, account: address, symbol: vector<u8>): u64 {
        primary_fungible_store::balance(account, metadata(admin, symbol))
    }

    /// Mint tokens for an existing asset
    public entry fun mint(
        admin: &signer,
        symbol: vector<u8>,
        to: address,
        amount: u64
    ) acquires FungibleAssetRegistry, ManagedFungibleAsset {
        let admin_address = signer::address_of(admin);
        assert_asset_exists(admin_address, copy symbol);
        let fa_address = fa_address(admin_address, copy symbol);
        let metadata = metadata(admin_address, copy symbol);
        let managed_fa = borrow_global<ManagedFungibleAsset>(fa_address);
        let to_wallet = primary_fungible_store::ensure_primary_store_exists(to, metadata);
        let fa = fungible_asset::mint(&managed_fa.mint_ref, amount);
        fungible_asset::deposit_with_ref(&managed_fa.transfer_ref, to_wallet, fa);
    }

    public entry fun transfer_fee(
        from: address,
        admin: address,
        amount: u64,
        symbol: vector<u8>
    ) acquires FungibleAssetRegistry, ManagedFungibleAsset {
        if (!exists<FungibleAssetRegistry>(admin)) {
            return;
        };

        let registry = borrow_global<FungibleAssetRegistry>(admin);
        if (!vector::contains(&registry.assets, &symbol)) {
            return;
        };

        assert!(amount > 0, error::invalid_argument(EINSUFFICIENT_BALANCE));
        let from_balance = get_balance(admin, from, copy symbol);
        if (from_balance < amount) {
            return;
        };
        let fa_address = fa_address(admin, copy symbol);
        let managed_fa = borrow_global<ManagedFungibleAsset>(fa_address);
        let metadata = metadata(admin, copy symbol);
        let from_wallet = primary_fungible_store::ensure_primary_store_exists(from, metadata);
        let admin_wallet = primary_fungible_store::ensure_primary_store_exists(admin, metadata);
        fungible_asset::transfer_with_ref(&managed_fa.transfer_ref, from_wallet, admin_wallet, amount);
    }


    public fun assert_asset_exists(admin: address, symbol: vector<u8>) acquires FungibleAssetRegistry {
        assert!(
            asset_exists(admin, symbol),
            error::not_found(EASSET_NOT_FOUND)
        );
    }

    #[view]
    fun asset_exists(admin: address, symbol: vector<u8>): bool acquires FungibleAssetRegistry {
        let registry = borrow_global<FungibleAssetRegistry>(admin);
            vector::contains(&registry.assets, &symbol)
    }

}

