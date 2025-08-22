// This module provides an interface to create, mint, and authorize transfers for stablecoin.
module cedra_framework::stablecoin {
    use std::vector;
    use std::signer;
    use std::option;
    use std::string::{Self, String};

    use cedra_framework::event;
    use cedra_framework::object::{Self, Object};
    use cedra_framework::fungible_asset::{Self, MintRef, TransferRef, Metadata};
    use cedra_framework::primary_fungible_store;

    friend cedra_framework::transaction_fee;
    friend cedra_framework::whitelist;

    /// Caller is not authorized to make this call
    const EUNAUTHORIZED: u64 = 1;
    // Not enought balance
    const EINSUFFICIENT_BALANCE: u64 = 2;
    /// Caller is already minter
    const EALREADY_MINTER: u64 = 3;

    #[resource_group_member(group = cedra_framework::object::ObjectGroup)]
    /// Resource to control fungible assets refs.
    struct Management has key {
        transfer_ref: TransferRef,
        mint_ref: MintRef
    }

    #[resource_group_member(group = cedra_framework::object::ObjectGroup)]
    /// Resource to control who can use fungible assets refs.
    struct Roles has key {
        admin: address,
        authorized_callers: vector<address>,
        master_minter: address,
        minters: vector<address>
    }

    #[event]
    struct Mint has drop, store {
        minter: address,
        to: address,
        amount: u64
    }

    /// Create a new fungible asset with associated roles and management.
    public entry fun create(
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

    /// Mint new tokens to the specified account. Caller be a minter.
    public entry fun mint(
        minter: &signer,
        creator_addr: address,
        symbol: vector<u8>,
        amount: u64
    ) acquires Roles, Management {
        if (amount == 0) { return };

        let minter_addr = signer::address_of(minter);
        let roles = borrow_global_mut<Roles>(asset_address(creator_addr, symbol));

        let is_auth = vector::contains(&roles.minters, &minter_addr);
        assert!(is_auth, EUNAUTHORIZED);

        let management = borrow_global<Management>(asset_address(creator_addr, symbol));

        fungible_asset::mint_to(
            &management.mint_ref,
            std::primary_fungible_store::ensure_primary_store_exists(
                minter_addr, get_metadata(creator_addr, symbol)
            ),
            amount
        );

        event::emit(Mint { minter: minter_addr, to: creator_addr, amount });
    }

    /// Add a new minter. Must be called by the master minter.
    public entry fun add_minter(
        creator: &signer, minter: address, symbol: vector<u8>
    ) acquires Roles {
        let creator_address = signer::address_of(creator);
        let roles = borrow_global_mut<Roles>(asset_address(creator_address, symbol));

        assert!(creator_address == roles.master_minter, EUNAUTHORIZED);
        assert!(!vector::contains(&roles.minters, &minter), EALREADY_MINTER);

        vector::push_back(&mut roles.minters, minter);
    }

    /// Add the account as an authorized caller.
    public entry fun update_authorized_caller(
        creator: &signer, authorized_caller: address, symbol: vector<u8>
    ) acquires Roles {
        let creator_address = signer::address_of(creator);
        let roles = borrow_global_mut<Roles>(asset_address(creator_address, symbol));

        assert!(creator_address == roles.master_minter, EUNAUTHORIZED);
        vector::push_back(&mut roles.authorized_callers, authorized_caller);
    }

    /// Transfer tokens with authorization check.
    public(friend) fun authorized_transfer(
        creator_addr: address,
        authorized_caller: address,
        from: address,
        to: address,
        symbol: vector<u8>,
        amount: u64
    ) acquires Roles, Management {
        if (amount == 0) { return };

        let asset_addr = object::object_address(&get_metadata(creator_addr, symbol));
        let from_balance = get_balance(creator_addr, from, copy symbol);
        assert!(from_balance >= amount, EINSUFFICIENT_BALANCE);

        let roles = borrow_global<Roles>(asset_addr);
        let management = borrow_global<Management>(asset_addr);

        let is_auth = vector::contains(&roles.authorized_callers, &authorized_caller);
        if (!is_auth) {
            return;
        };

        primary_fungible_store::transfer_with_ref(
            &management.transfer_ref, from, to, amount
        );
    }

    public(friend) fun asset_deployed(owner: address, symbol: vector<u8>): bool {
        exists<Roles>(asset_address(owner, symbol))
    }

    fun asset_address(owner: address, symbol: vector<u8>): address {
        object::create_object_address(&owner, symbol)
    }

    fun assert_is_admin(admin: address, symbol: vector<u8>) acquires Roles {
        let roles = borrow_global<Roles>(asset_address(admin, symbol));
        assert!(@admin == roles.admin, EUNAUTHORIZED);
    }

    fun get_metadata(creator: address, symbol: vector<u8>): Object<Metadata> {
        object::address_to_object<Metadata>(asset_address(creator, symbol))
    }

    #[view]
    public fun get_authorized_callers(
        creator_address: address, symbol: vector<u8>
    ): vector<address> acquires Roles {
        let asset_addr = asset_address(creator_address, symbol);
        borrow_global<Roles>(asset_addr).authorized_callers
    }

    #[view]
    public fun get_balance(
        admin: address, account: address, symbol: vector<u8>
    ): u64 {
        primary_fungible_store::balance(account, get_metadata(admin, symbol))
    }
}
