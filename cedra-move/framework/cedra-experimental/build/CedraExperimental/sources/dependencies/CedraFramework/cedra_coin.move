/// This module defines a minimal and generic Coin and Balance.
/// modified from https://github.com/move-language/move/tree/main/language/documentation/tutorial
module cedra_framework::cedra_coin {
    use std::error;
    use std::signer;
    use std::string;
    use std::vector;
    use std::option::{Self, Option};

    use cedra_framework::coin::{Self, BurnCapability, MintCapability};
    use cedra_framework::system_addresses;

    friend cedra_framework::genesis;

    /// Account does not have mint capability
    const ENO_CAPABILITIES: u64 = 1;
    /// Mint capability has already been delegated to this specified address
    const EALREADY_DELEGATED: u64 = 2;
    /// Cannot find delegation of mint capability to this account
    const EDELEGATION_NOT_FOUND: u64 = 3;

    struct CedraCoin has key {}

    struct MintCapStore has key {
        mint_cap: MintCapability<CedraCoin>,
    }

    /// Delegation token created by delegator and can be claimed by the delegatee as MintCapability.
    struct DelegatedMintCapability has store {
        to: address
    }

    /// The container stores the current pending delegations.
    struct Delegations has key {
        inner: vector<DelegatedMintCapability>,
    }

    /// Can only called during genesis to initialize the Cedra coin.
    public(friend) fun initialize(cedra_framework: &signer): (BurnCapability<CedraCoin>, MintCapability<CedraCoin>) {
        system_addresses::assert_cedra_framework(cedra_framework);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<CedraCoin>(
            cedra_framework,
            string::utf8(b"Cedra Coin"),
            string::utf8(b"Cedra"),
            8, // decimals
            true, // monitor_supply
        );

        // Cedra framework needs mint cap to mint coins to initial validators. This will be revoked once the validators
        // have been initialized.
        move_to(cedra_framework, MintCapStore { mint_cap });

        coin::destroy_freeze_cap(freeze_cap);
        (burn_cap, mint_cap)
    }

    public fun has_mint_capability(account: &signer): bool {
        exists<MintCapStore>(signer::address_of(account))
    }

    /// Only called during genesis to destroy the cedra framework account's mint capability once all initial validators
    /// and accounts have been initialized during genesis.
    public(friend) fun destroy_mint_cap(cedra_framework: &signer) acquires MintCapStore {
        system_addresses::assert_cedra_framework(cedra_framework);
        let MintCapStore { mint_cap } = move_from<MintCapStore>(@cedra_framework);
        coin::destroy_mint_cap(mint_cap);
    }

    /// Can only be called during genesis for tests to grant mint capability to cedra framework and core resources
    /// accounts.
    /// Expects account and Cedra store to be registered before calling.
    public(friend) fun configure_accounts_for_test(
        cedra_framework: &signer,
        core_resources: &signer,
        mint_cap: MintCapability<CedraCoin>,
    ) {
        system_addresses::assert_cedra_framework(cedra_framework);

        // Mint the core resource account CedraCoin for gas so it can execute system transactions.
        let coins = coin::mint<CedraCoin>(
            18446744073709551615,
            &mint_cap,
        );
        coin::deposit<CedraCoin>(signer::address_of(core_resources), coins);

        move_to(core_resources, MintCapStore { mint_cap });
        move_to(core_resources, Delegations { inner: vector::empty() });
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Create new coins and deposit them into dst_addr's account.
    public entry fun mint(
        account: &signer,
        dst_addr: address,
        amount: u64,
    ) acquires MintCapStore {
        let account_addr = signer::address_of(account);

        assert!(
            exists<MintCapStore>(account_addr),
            error::not_found(ENO_CAPABILITIES),
        );

        let mint_cap = &borrow_global<MintCapStore>(account_addr).mint_cap;
        let coins_minted = coin::mint<CedraCoin>(amount, mint_cap);
        coin::deposit<CedraCoin>(dst_addr, coins_minted);
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Create delegated token for the address so the account could claim MintCapability later.
    public entry fun delegate_mint_capability(account: signer, to: address) acquires Delegations {
        system_addresses::assert_core_resource(&account);
        let delegations = &mut borrow_global_mut<Delegations>(@core_resources).inner;
        vector::for_each_ref(delegations, |element| {
            let element: &DelegatedMintCapability = element;
            assert!(element.to != to, error::invalid_argument(EALREADY_DELEGATED));
        });
        vector::push_back(delegations, DelegatedMintCapability { to });
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Claim the delegated mint capability and destroy the delegated token.
    public entry fun claim_mint_capability(account: &signer) acquires Delegations, MintCapStore {
        let maybe_index = find_delegation(signer::address_of(account));
        assert!(option::is_some(&maybe_index), EDELEGATION_NOT_FOUND);
        let idx = *option::borrow(&maybe_index);
        let delegations = &mut borrow_global_mut<Delegations>(@core_resources).inner;
        let DelegatedMintCapability { to: _ } = vector::swap_remove(delegations, idx);

        // Make a copy of mint cap and give it to the specified account.
        let mint_cap = borrow_global<MintCapStore>(@core_resources).mint_cap;
        move_to(account, MintCapStore { mint_cap });
    }

    fun find_delegation(addr: address): Option<u64> acquires Delegations {
        let delegations = &borrow_global<Delegations>(@core_resources).inner;
        let i = 0;
        let len = vector::length(delegations);
        let index = option::none();
        while (i < len) {
            let element = vector::borrow(delegations, i);
            if (element.to == addr) {
                index = option::some(i);
                break
            };
            i = i + 1;
        };
        index
    }

    #[test_only]
    use cedra_framework::account;
    #[test_only]
    use cedra_framework::aggregator_factory;
    #[test_only]
    use cedra_framework::fungible_asset::FungibleAsset;

    #[test_only]
    public fun mint_apt_fa_for_test(amount: u64): FungibleAsset acquires MintCapStore {
        ensure_initialized_with_apt_fa_metadata_for_test();
        coin::coin_to_fungible_asset(
            coin::mint(
                amount,
                &borrow_global<MintCapStore>(@cedra_framework).mint_cap
            )
        )
    }

    #[test_only]
    public fun ensure_initialized_with_apt_fa_metadata_for_test() {
        let cedra_framework = account::create_signer_for_test(@cedra_framework);
        if (!exists<MintCapStore>(@cedra_framework)) {
            if (!aggregator_factory::aggregator_factory_exists_for_testing()) {
                aggregator_factory::initialize_aggregator_factory_for_test(&cedra_framework);
            };
            let (burn_cap, mint_cap) = initialize(&cedra_framework);
            coin::destroy_burn_cap(burn_cap);
            coin::destroy_mint_cap(mint_cap);
        };
        coin::create_coin_conversion_map(&cedra_framework);
        coin::create_pairing<CedraCoin>(&cedra_framework);
    }

    #[test_only]
    public fun initialize_for_test(cedra_framework: &signer): (BurnCapability<CedraCoin>, MintCapability<CedraCoin>) {
        aggregator_factory::initialize_aggregator_factory_for_test(cedra_framework);
        let (burn_cap, mint_cap) = initialize(cedra_framework);
        coin::create_coin_conversion_map(cedra_framework);
        coin::create_pairing<CedraCoin>(cedra_framework);
        (burn_cap, mint_cap)
    }

    // This is particularly useful if the aggregator_factory is already initialized via another call path.
    #[test_only]
    public fun initialize_for_test_without_aggregator_factory(
        cedra_framework: &signer
    ): (BurnCapability<CedraCoin>, MintCapability<CedraCoin>) {
        let (burn_cap, mint_cap) = initialize(cedra_framework);
        coin::create_coin_conversion_map(cedra_framework);
        coin::create_pairing<CedraCoin>(cedra_framework);
        (burn_cap, mint_cap)
    }
}
