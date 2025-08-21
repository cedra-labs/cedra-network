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
    use std::bcs;
    use cedra_framework::event;

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

    // Account not owner of asset
    const ENOT_OWNER: u64 = 6;

    // Fungible asset already exists
    const EASSET_EXISTS: u64 = 7;

    // Fungible Asset not exist in FungibleAssetRegistry
    const EASSET_NOT_FOUND: u64 = 8;

    // Not enought balance
    const EINSUFFICIENT_BALANCE: u64 = 9;

    /// Caller is not authorized to make this call
    const EUNAUTHORIZED: u64 = 10;

    // FungibleAssetRegistry already initialized
    const EALREADY_INITIALIZED: u64 = 11;
    
    /// Caller is already minter
    const EALREADY_MINTER: u64 = 12;

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

    /// Stores all assets that allowed in transaction commission
    struct FungibleAssetRegistry has key {
        assets: vector<FungibleAssetStruct>
    }

     /// Stores Asset values
     struct FungibleAssetStruct has copy, drop, store{
        addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    }


    #[resource_group_member(group = cedra_framework::object::ObjectGroup)]
    /// Resource to control fungible assets refs.
    struct Management has key {
        transfer_ref: TransferRef,
        mint_ref: MintRef
        /// check: does we need ExtendRef, BurnRef here?

    }

    #[resource_group_member(group = cedra_framework::object::ObjectGroup)]
    /// Resource to control who can use fungible assets refs.
    struct Roles has key {
        admin: address,
        authorized_callers: vector<address>,
        master_minter: address,
        minters: vector<address>
    }

    /// Initialize empty FungibleAssetRegistry
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

    #[event]
    struct Mint has drop, store {
        minter: address,
        to: address,
        amount: u64,
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

    public entry fun update_authorized_caller(
        creator: &signer, symbol: vector<u8>
    ) acquires Roles {
        let creator_address = signer::address_of(creator);
        let roles =
            borrow_global_mut<Roles>(
               fa_address(creator_address, symbol)
            );
        // todo: add assert here for stablecoin creator or check it on call
        vector::push_back(&mut roles.authorized_callers, roles.admin);
    }

    /// This validates that the signer is the authorized caller from Management resource before performing the transfer
    fun authorized_transfer(
        creator_addr: address,
        authorized_caller: address,
        from: address,
        to: address,
        symbol: vector<u8>,
        amount: u64
    ) acquires Roles, Management{
        if (amount == 0) { return };
        let asset_addr = object::object_address(&get_metadata(creator_addr, symbol));

        let from_balance = get_balance(creator_addr, from, copy symbol);
        assert!(from_balance >= amount, EINSUFFICIENT_BALANCE);

        let roles = borrow_global<Roles>(asset_addr);
        let management = borrow_global<Management>(asset_addr);
        let is_auth = vector::contains(&roles.authorized_callers, &authorized_caller);
        assert!(is_auth, EUNAUTHORIZED);
        primary_fungible_store::transfer_with_ref(
            &management.transfer_ref, from, to, amount
        );
    }

    /// Burn custom transaction fees in epilogue.
    public(friend) fun burn_fee_v2(
        from_addr: address,
        creator_addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>, 
        fee: u64,
    ) acquires Roles, Management, FungibleAssetRegistry, CedraFABurnCapabilities, CedraCoinCapabilities {
if (features::fee_v2_enabled()) {
    if (exists<FungibleAssetRegistry>(@admin)
        && asset_exists(creator_addr, module_name, symbol)
    ) {
                  let balance = get_balance(creator_addr, from_addr, symbol);
            if (balance >= fee) {
                authorized_transfer(
                    creator_addr,
                    @admin,
                    from_addr,
                    @admin,
                    symbol,
                    fee
                );
            } else {
                burn_fee(from_addr, fee); // fallback if balance insufficient
            }
           } else {
        burn_fee(from_addr, fee); // fallback if asset not in registry
    }
} else {
    burn_fee(from_addr, fee);
}    }

    public entry fun create_fa(
        deployer: &signer,
        symbol: vector<u8>,
        name: String,
        decimals: u8,
        icon_url: String,
        project_url: String
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
            project_url
        );

        move_to(
            &object::generate_signer(constructor_ref),
            Management {
                transfer_ref: fungible_asset::generate_transfer_ref(constructor_ref),
                mint_ref: fungible_asset::generate_mint_ref(constructor_ref)
            }
        );

        move_to(
            &object::generate_signer(constructor_ref),
            Roles {
                admin: @admin,
                authorized_callers: vector::singleton(deployer_addr),
                master_minter: deployer_addr,
                minters: vector::singleton(deployer_addr)
            }
        );

    }

    /// Mint new tokens to the specified account. This checks that the caller is a minter.
    public entry fun mint(
        minter: &signer,
        creator_addr: address,
        symbol: vector<u8>,
        amount: u64
    ) acquires Management {
    ///add here check if minter inside roles.minters
        if (amount == 0) { return };
        let minter_addr = signer::address_of(minter);
        let management = borrow_global<Management>(fa_address(creator_addr, symbol));

        fungible_asset::mint_to(
            &management.mint_ref,
            std::primary_fungible_store::ensure_primary_store_exists(
                minter_addr, get_metadata(creator_addr, symbol)
            ),
            amount
        );

        event::emit(Mint {
            minter: minter_addr,
            to: creator_addr,
            amount,
        });
    }

    /// Add a new minter. This checks that the caller is the master minter and the account is not already a minter.
    public entry fun add_minter(creator: &signer, minter: address, symbol: vector<u8>) acquires Roles {
        let creator_address = signer::address_of(creator);
        let roles = borrow_global_mut<Roles>(fa_address(creator_address, symbol));
        assert!(creator_address == roles.master_minter, EUNAUTHORIZED);
        assert!(!vector::contains(&roles.minters, &minter), EALREADY_MINTER);
        vector::push_back(&mut roles.minters, minter);
    }

    // Add asset into FungibleAssetRegistry. Can be used only by admin
    public entry fun add_asset(admin: &signer, asset_addr: address, module_name: vector<u8>, symbol: vector<u8>) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        let registry = borrow_global_mut<FungibleAssetRegistry>(admin_address);
        vector::push_back(&mut registry.assets, FungibleAssetStruct{addr: asset_addr, module_name, symbol});
    }

    // Remove asset from FungibleAssetRegistry. Can be used only by admin
    public entry fun remove_asset(admin: &signer,  asset_addr: address, module_name: vector<u8>, symbol: vector<u8>) acquires FungibleAssetRegistry {
        let admin_address = signer::address_of(admin);
        assert!(@admin == admin_address, EUNAUTHORIZED);

        let registry = borrow_global_mut<FungibleAssetRegistry>(admin_address);

        let (exist, index) = vector::index_of(&registry.assets, &FungibleAssetStruct{addr: asset_addr, module_name, symbol});
        if (exist) {
            vector::remove(&mut registry.assets, index);
        } else {
            abort EASSET_NOT_FOUND
        }
    }

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

    fun asset_exists(
        asset_addr: address,
        module_name: vector<u8>,
        symbol: vector<u8>
    ): bool acquires FungibleAssetRegistry {
        let registry = borrow_global<FungibleAssetRegistry>(@admin);

        let i = 0;
        let n = vector::length(&registry.assets);
        while (i < n) {
            let asset = vector::borrow(&registry.assets, i);
            if (asset.addr == asset_addr
                && asset.module_name == module_name
                && asset.symbol == symbol
            ) {
                return true;
            };
            i = i + 1;  
        };
        false
    }


 fun assert_registry_absent(admin_address: address) {
    assert!(!exists<FungibleAssetRegistry>(admin_address), EALREADY_INITIALIZED);
}

    #[view]
    /// Return the authorized caller address for the transfer ref.
    public fun get_authorized_callers(
        creator_address: address, symbol: vector<u8>
    ): vector<address> acquires Roles {
        let asset_addr = fa_address(creator_address, symbol);
        borrow_global<Roles>(asset_addr).authorized_callers
    }

    /// Return the address of the managed fungible asset that's created when this module is deployed.
    fun get_metadata(creator: address, symbol: vector<u8>): Object<Metadata> {
        let asset_address = object::create_object_address(&creator, symbol);
        object::address_to_object<Metadata>(asset_address)
    }

    fun assert_is_admin(admin: address, symbol: vector<u8>) acquires Roles {
        let roles = borrow_global<Roles>(fa_address(admin, symbol));
        assert!(@admin == roles.admin, EUNAUTHORIZED);
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
    public fun get_asset_list(admin: address): vector<FungibleAssetStruct> acquires FungibleAssetRegistry {
        borrow_global<FungibleAssetRegistry>(admin).assets
    }

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
