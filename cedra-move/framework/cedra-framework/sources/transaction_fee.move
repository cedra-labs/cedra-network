// This module provides an interface to burn or collect and redistribute transaction fees.
module cedra_framework::transaction_fee {
    use cedra_framework::coin::{Self, AggregatableCoin, BurnCapability, MintCapability};
    use cedra_framework::cedra_account;
    use cedra_framework::cedra_coin::CedraCoin;
    use cedra_framework::fungible_asset::{Self, MintRef, TransferRef, BurnRef, Metadata};
    use cedra_framework::object::{Self, Object};
    use cedra_framework::system_addresses;
    use cedra_framework::primary_fungible_store;
    use std::error;
    use std::vector;
    use std::string::{Self, String};
    use std::features;
    use std::option::{Self, Option};
    use std::signer;
    use cedra_framework::event;
    use std::math64;
    use std::string::utf8;


    friend cedra_framework::block;
    friend cedra_framework::genesis;
    friend cedra_framework::reconfiguration;
    friend cedra_framework::transaction_validation;

    /// Gas fees are already being collected and the struct holding
    /// information about collected amounts is already published.
    const EALREADY_COLLECTING_FEES: u64 = 1;

    /// The burn percentage is out of range [0, 100].
    const EINVALID_BURN_PERCENTAGE: u64 = 3;

    /// No longer supported.
    const ENO_LONGER_SUPPORTED: u64 = 4;

    const EFA_GAS_CHARGING_NOT_ENABLED: u64 = 5;

    /// ------------
    // fee transfer errors
    const ENOT_OWNER: u64 = 6;
    const EASSET_EXISTS: u64 = 7;
    const EASSET_NOT_FOUND: u64 = 8;
    const EINSUFFICIENT_BALANCE: u64 = 9;
    
    /// the caller must be authorized
    const EUNAUTHORIZED: u64 = 10;
    const DECIMALS: u64 = 8;


    /// Stores burn capability to burn the gas fees.
    struct CedraCoinCapabilities has key {
        burn_cap: BurnCapability<CedraCoin>
    }

    /// Stores burn capability to burn the gas fees.
    struct CedraFABurnCapabilities has key {
        burn_ref: BurnRef
    }

    /// Stores mint capability to mint the refunds.
    struct CedraCoinMintCapability has key {
        mint_cap: MintCapability<CedraCoin>
    }

    struct FungibleAssetRegistry has key {
        assets: vector<String>
    }

       #[resource_group_member(group = cedra_framework::object::ObjectGroup)]
    /// Resource to control the transfer ref of fungible assets.
    struct Info has key {
        authorized_caller_addr: vector<address>,
        transfer_ref: TransferRef
    }


    

    /// Per-asset management resources
    struct ManagedFungibleAsset has key {
        mint_ref: MintRef,
        transfer_ref: TransferRef,
        burn_ref: BurnRef,
        asset_identifier: String
    }

    /// Initialize the factory
    // Todo: init_module don't work here because we use governance
    // we have 2 options: change governance publish_tx() or call this by hand
    // and create guards to lock after first use
    public entry fun init_registry(admin: &signer) {
        move_to(
            admin,
            FungibleAssetRegistry {
                assets: vector::empty<String>()
            }
        );
    }

    #[event]
    /// Breakdown of fee charge and refund for a transaction.
    /// The structure is:
    ///
    /// - Net charge or refund (not in the statement)
    ///    - total charge: total_charge_gas_units, matches `gas_used` in the on-chain `TransactionInfo`.
    ///      This is the sum of the sub-items below. Notice that there's potential precision loss when
    ///      the conversion between internal and external gas units and between native token and gas
    ///      units, so it's possible that the numbers don't add up exactly. -- This number is the final
    ///      charge, while the break down is merely informational.
    ///        - gas charge for execution (CPU time): `execution_gas_units`
    ///        - gas charge for IO (storage random access): `io_gas_units`
    ///        - storage fee charge (storage space): `storage_fee_octas`, to be included in
    ///          `total_charge_gas_unit`, this number is converted to gas units according to the user
    ///          specified `gas_unit_price` on the transaction.
    ///    - storage deletion refund: `storage_fee_refund_octas`, this is not included in `gas_used` or
    ///      `total_charge_gas_units`, the net charge / refund is calculated by
    ///      `total_charge_gas_units` * `gas_unit_price` - `storage_fee_refund_octas`.
    ///
    /// This is meant to emitted as a module event.
    struct FeeStatement has drop, store {
        /// Total gas charge.
        total_charge_gas_units: u64,
        /// Execution gas charge.
        execution_gas_units: u64,
        /// IO gas charge.
        io_gas_units: u64,
        /// Storage fee charge.
        storage_fee_octas: u64,
        /// Storage fee refund.
        storage_fee_refund_octas: u64
    }

    #[event]
    /// Breakdown of fee charge and refund for a transaction.
    /// The structure is:
    ///
    /// - Net charge or refund (not in the statement)
    ///    - total charge: total_charge_gas_units, matches `gas_used` in the on-chain `TransactionInfo`.
    ///      This is the sum of the sub-items below. Notice that there's potential precision loss when
    ///      the conversion between internal and external gas units and between native token and gas
    ///      units, so it's possible that the numbers don't add up exactly. -- This number is the final
    ///      charge, while the break down is merely informational.
    ///        - gas charge for execution (CPU time): `execution_gas_units`
    ///        - gas charge for IO (storage random access): `io_gas_units`
    ///        - storage fee charge (storage space): `storage_fee_octas`, to be included in
    ///          `total_charge_gas_unit`, this number is converted to gas units according to the user
    ///          specified `gas_unit_price` on the transaction.
    ///    - storage deletion refund: `storage_fee_refund_octas`, this is not included in `gas_used` or
    ///      `total_charge_gas_units`, the net charge / refund is calculated by
    ///      `total_charge_gas_units` * `gas_unit_price` - `storage_fee_refund_octas`.
    ///
    /// This is meant to emitted as a module event.
    struct CustomFeeStatement has drop, store {
        /// Total gas charge.
        total_charge_gas_units: u64,
        /// Execution gas charge.
        execution_gas_units: u64,
        /// IO gas charge.
        io_gas_units: u64,
        /// Storage fee charge.
        storage_fee_octas: u64,
        /// Storage fee refund.
        storage_fee_refund_octas: u64
    }

    /// Burn transaction fees in epilogue.
    public(friend) fun burn_fee(
        account: address, fee: u64
    ) acquires CedraFABurnCapabilities, CedraCoinCapabilities {
        if (exists<CedraFABurnCapabilities>(@cedra_framework)) {
            let burn_ref =
                &borrow_global<CedraFABurnCapabilities>(@cedra_framework).burn_ref;
            cedra_account::burn_from_fungible_store_for_gas(burn_ref, account, fee);
        } else {
            let burn_cap =
                &borrow_global<CedraCoinCapabilities>(@cedra_framework).burn_cap;
            if (features::operations_default_to_fa_apt_store_enabled()) {
                let (burn_ref, burn_receipt) = coin::get_paired_burn_ref(burn_cap);
                cedra_account::burn_from_fungible_store_for_gas(&burn_ref, account, fee);
                coin::return_paired_burn_ref(burn_ref, burn_receipt);
            } else {
                coin::burn_from_for_gas<CedraCoin>(account, fee, burn_cap);
            };
        };
    }

    public entry fun update_authorized_caller(symbol: vector<u8>) acquires Info {
        let info = borrow_global_mut<Info>(object::object_address(&get_metadata(symbol)));
        // todo: add assert here for stablecoin creator
        let old_authorized_caller = info.authorized_caller_addr;
            vector::push_back(
            &mut info.authorized_caller_addr,
            @admin
        );
    }

    /// This validates that the signer is the authorized caller from Info resource before performing the transfer
    public entry fun authorized_transfer(
        authorized_caller: address,
        from: address,
        to: address,
        symbol: vector<u8>,
        amount: u64
    ) acquires Info {
        let info = borrow_global<Info>(object::object_address(&get_metadata(symbol)));
        let is_auth = vector::contains(&info.authorized_caller_addr, &authorized_caller);
        assert!(is_auth, EUNAUTHORIZED);        

        primary_fungible_store::transfer_with_ref(
            &info.transfer_ref,
            from,
            to,
            amount
        );
    }

    


    /// Burn custom transaction fees in epilogue.
    public(friend) fun burn_fee_v2 (
        account: address,
        fee: u64,
        fa_address: address,
        fa_module: vector<u8>,
        fa_symbol: vector<u8>
    ) acquires Info{
        // let registry = borrow_global<FungibleAssetRegistry>(@admin);
        // let symbol_str = string::utf8(fa_symbol);
        // let module_str = string::utf8(fa_module);

        // if (fa_address
        //     == @0xcf457e2e62739e7cc6d2b906acba3f17a708e0b98ed13518b221f79026dcd7b4
        //     && module_str == string::utf8(b"usdt")
        //     && symbol_str == string::utf8(b"USDT")) {
            // Find asset index with proper error handling
            // let index = find_asset_index(&registry.assets, symbol_str);

            // let asset_entry_ref = vector::borrow(&registry.assets, index);
            // let transfer_fn = *&asset_entry_ref.transfer_fn;
              if (features::fee_v2_enabled()) {
                authorized_transfer(
                    @admin,  // &signer (must be authorized)
                    account,            // from: address
                    @admin,             // to: address (your fee admin)
                    fa_symbol,
                    fee                 // amount: u64
            );
            }            
        // }
    }

    public entry fun create_fa(
        deployer: &signer,
        symbol: vector<u8>,
        name: String,
        decimals: u8,
        icon_url: String,
        project_url: String,
    ) {
        let deployer_addr = signer::address_of(deployer);
        let constructor_ref = &object::create_named_object(deployer, symbol);
        primary_fungible_store::create_primary_store_enabled_fungible_asset(
            constructor_ref,
            option::none(),
            name,
            string::utf8(copy symbol),
            decimals,
            icon_url,
            project_url,
        );

            fungible_asset::mint_to(
            &fungible_asset::generate_mint_ref(constructor_ref),
            std::primary_fungible_store::ensure_primary_store_exists(deployer_addr, get_metadata(symbol)),
            1_000_000_000 * (math64::pow(10, 8))
        );



        move_to(
            &object::generate_signer(constructor_ref),
            Info {
            authorized_caller_addr: vector::singleton(deployer_addr),
                transfer_ref: fungible_asset::generate_transfer_ref(constructor_ref) 
            },
        );
    }

    fun find_asset_index(assets: &vector<String>, target: String): u64 {
        let i = 0;
        let len = vector::length(assets);
        while (i < len) {
            let asset_id = *vector::borrow(assets, i);
            if (asset_id == target) {
                return i
            };
            i = i + 1;
        };
        len
    }

    public entry fun add_asset(
        admin: &signer, asset_id: String, transfer_fn: String
    ) acquires FungibleAssetRegistry {
        let registry = borrow_global_mut<FungibleAssetRegistry>(signer::address_of(admin));
        vector::push_back(
            &mut registry.assets,
             asset_id
        );
    }

    // /// Remove an asset by index
    // public entry fun remove_asset(
    //     registry: &mut FungibleAssetRegistry, index: u64
    // ) {
    //     assert!(index < vector::length(&registry.assets), 0);
    //     vector::remove(&mut registry.assets, index);
    // }

    /// Get an asset entry by index
    public fun get_asset(registry: &FungibleAssetRegistry, index: u64): &String{
        assert!(index < vector::length(&registry.assets), 0);
         vector::borrow(&registry.assets, index)
    }

    // public entry fun transfer_fee(
    //     from: address,
    //     admin: address,
    //     amount: u64,
    //     symbol: vector<u8>
    // ) acquires FungibleAssetRegistry, ManagedFungibleAsset {
    //     if (!exists<FungibleAssetRegistry>(admin)) {
    //         return;
    //     };

    //     let registry = borrow_global<FungibleAssetRegistry>(admin);
    //     // if (!vector::contains(&registry.assets, &symbol)) {
    //     //     return;
    //     // };

    //     assert!(amount > 0, error::invalid_argument(EINSUFFICIENT_BALANCE));
    //     let from_balance = get_balance(admin, from, copy symbol);
    //     if (from_balance < amount) {
    //         return;
    //     };
    //     let fa_address = fa_address(admin, copy symbol);
    //     let managed_fa = borrow_global<ManagedFungibleAsset>(fa_address);
    //     let metadata = metadata(admin, copy symbol);
    //     let from_wallet =
    //         primary_fungible_store::ensure_primary_store_exists(from, metadata);
    //     let admin_wallet =
    //         primary_fungible_store::ensure_primary_store_exists(admin, metadata);
    //     fungible_asset::transfer_with_ref(
    //         &managed_fa.transfer_ref,
    //         from_wallet,
    //         admin_wallet,
    //         amount
    //     );
    // }

    /// Mint refund in epilogue.
    public(friend) fun mint_and_refund(
        account: address, refund: u64
    ) acquires CedraCoinMintCapability {
        let mint_cap = &borrow_global<CedraCoinMintCapability>(@cedra_framework).mint_cap;
        let refund_coin = coin::mint(refund, mint_cap);
        coin::deposit_for_gas_fee(account, refund_coin);
    }

    /// Only called during genesis.
    public(friend) fun store_cedra_coin_burn_cap(
        cedra_framework: &signer, burn_cap: BurnCapability<CedraCoin>
    ) {
        system_addresses::assert_cedra_framework(cedra_framework);

        if (features::operations_default_to_fa_apt_store_enabled()) {
            let burn_ref = coin::convert_and_take_paired_burn_ref(burn_cap);
            move_to(cedra_framework, CedraFABurnCapabilities { burn_ref });
        } else {
            move_to(cedra_framework, CedraCoinCapabilities { burn_cap })
        }
    }

    public entry fun convert_to_cedra_fa_burn_ref(
        cedra_framework: &signer
    ) acquires CedraCoinCapabilities {
        assert!(
            features::operations_default_to_fa_apt_store_enabled(),
            EFA_GAS_CHARGING_NOT_ENABLED
        );
        system_addresses::assert_cedra_framework(cedra_framework);
        let CedraCoinCapabilities { burn_cap } =
            move_from<CedraCoinCapabilities>(signer::address_of(cedra_framework));
        let burn_ref = coin::convert_and_take_paired_burn_ref(burn_cap);
        move_to(cedra_framework, CedraFABurnCapabilities { burn_ref });
    }

    /// Only called during genesis.
    public(friend) fun store_cedra_coin_mint_cap(
        cedra_framework: &signer, mint_cap: MintCapability<CedraCoin>
    ) {
        system_addresses::assert_cedra_framework(cedra_framework);
        move_to(cedra_framework, CedraCoinMintCapability { mint_cap })
    }

    // Called by the VM after epilogue.
    fun emit_fee_statement(fee_statement: FeeStatement) {
        event::emit(fee_statement)
    }

    // Called by the VM after epilogue.
    fun emit_custom_fee_statement(
        custom_fee_statement: CustomFeeStatement
    ) {
        event::emit(custom_fee_statement)
    }

     #[view]
     fun asset_exists(admin: address, asset_id: String): bool acquires FungibleAssetRegistry {
         let registry = borrow_global<FungibleAssetRegistry>(admin);
         vector::contains(&registry.assets, &asset_id)
     }
    

        #[view]
    /// Return the authorized caller address for the transfer ref.
    public fun get_authorized_callers(symbol: vector<u8>): vector<address> acquires Info {
        let asset_addr = object::object_address(&get_metadata(symbol));
        borrow_global<Info>(asset_addr).authorized_caller_addr
    }

       #[view]
    /// Return the address of the managed fungible asset that's created when this module is deployed.
    public fun get_metadata(_symbol: vector<u8>): Object<Metadata> {
        let asset_address = object::create_object_address(&@creator, b"USDT");
        object::address_to_object<Metadata>(asset_address)
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

    // #[view]
    // // get list of fungible assets registered in FungibleAssetRegistry
    // public fun get_asset_list(admin: address): vector<String> acquires FungibleAssetRegistry {
    //     borrow_global<FungibleAssetRegistry>(admin).assets
    // }

    #[view]
    // get balance of fungible asset for account
    public fun get_balance(
        admin: address, account: address, symbol: vector<u8>
    ): u64 {
        primary_fungible_store::balance(account, metadata(admin, symbol))
    }

    // DEPRECATED section:

    #[deprecated]
    /// DEPRECATED: Stores information about the block proposer and the amount of fees
    /// collected when executing the block.
    struct CollectedFeesPerBlock has key {
        amount: AggregatableCoin<CedraCoin>,
        proposer: Option<address>,
        burn_percentage: u8
    }

    #[deprecated]
    /// DEPRECATED
    public fun initialize_fee_collection_and_distribution(
        _cedra_framework: &signer, _burn_percentage: u8
    ) {
        abort error::not_implemented(ENO_LONGER_SUPPORTED)
    }

    #[deprecated]
    /// DEPRECATED
    public fun upgrade_burn_percentage(
        _cedra_framework: &signer, _new_burn_percentage: u8
    ) {
        abort error::not_implemented(ENO_LONGER_SUPPORTED)
    }

    #[deprecated]
    public fun initialize_storage_refund(_: &signer) {
        abort error::not_implemented(ENO_LONGER_SUPPORTED)
    }
}
