spec cedra_framework::cedra_coin {
    /// <high-level-req>
    /// No.: 1
    /// Requirement: The native token, Cedra, must be initialized during genesis.
    /// Criticality: Medium
    /// Implementation: The initialize function is only called once, during genesis.
    /// Enforcement: Formally verified via [high-level-req-1](initialize).
    ///
    /// No.: 2
    /// Requirement: The Cedra coin may only be created exactly once.
    /// Criticality: Medium
    /// Implementation: The initialization function may only be called once.
    /// Enforcement: Enforced through the [https://github.com/cedra-labs/cedra-network/blob/main/cedra-move/framework/cedra-framework/sources/coin.move](coin)
    /// module, which has been audited.
    ///
    /// No.: 3
    /// Requirement: The abilities to mint Cedra tokens should be transferable, duplicatable, and destroyable.
    /// Criticality: High
    /// Implementation: The MintCapability struct has the copy and store abilities. This means that it can be duplicated
    /// and stored in different object wrappers (such as MintCapStore). This capability is tested against the
    /// destroy_mint_cap and claim_mint_capability functions.
    /// Enforcement: Verified via [high-level-req-3](initialize).

    /// No.: 4
    /// Requirement: Any type of operation on the Cedra coin should fail if the user has not registered for the coin.
    /// Criticality: Medium
    /// Implementation: Coin operations may succeed only on valid user coin registration.
    /// Enforcement: Enforced through the [https://github.com/cedra-labs/cedra-network/blob/main/cedra-move/framework/cedra-framework/sources/coin.move](coin)
    /// module, which has been audited.
    /// </high-level-req>
    ///
    spec module {
        pragma verify = true;
        pragma aborts_if_is_partial;
    }

    spec initialize(cedra_framework: &signer): (BurnCapability<CedraCoin>, MintCapability<CedraCoin>) {
        use cedra_framework::aggregator_factory;
        use cedra_framework::permissioned_signer;

        pragma verify = false;

        aborts_if permissioned_signer::spec_is_permissioned_signer(cedra_framework);
        let addr = signer::address_of(cedra_framework);
        aborts_if addr != @cedra_framework;
        aborts_if !string::spec_internal_check_utf8(b"Cedra Coin");
        aborts_if !string::spec_internal_check_utf8(b"Cedra");
        aborts_if exists<MintCapStore>(addr);
        aborts_if exists<coin::CoinInfo<CedraCoin>>(addr);
        aborts_if !exists<aggregator_factory::AggregatorFactory>(addr);
        /// [high-level-req-1]
        ensures exists<MintCapStore>(addr);
        // property 3: The abilities to mint Cedra tokens should be transferable, duplicatable, and destroyable.
        /// [high-level-req-3]
        ensures global<MintCapStore>(addr).mint_cap ==  MintCapability<CedraCoin> {};
        ensures exists<coin::CoinInfo<CedraCoin>>(addr);
        ensures result_1 == BurnCapability<CedraCoin> {};
        ensures result_2 == MintCapability<CedraCoin> {};
    }

    spec destroy_mint_cap {
        let addr = signer::address_of(cedra_framework);
        aborts_if addr != @cedra_framework;
        aborts_if !exists<MintCapStore>(@cedra_framework);
    }

    // Test function, not needed verify.
    spec configure_accounts_for_test {
        pragma verify = false;
    }

    // Only callable in tests and testnets. not needed verify.
    spec mint(
        account: &signer,
        dst_addr: address,
        amount: u64,
    ) {
        pragma verify = false;
    }

    // Only callable in tests and testnets. not needed verify.
    spec delegate_mint_capability {
        pragma verify = false;
    }

    // Only callable in tests and testnets. not needed verify.
    spec claim_mint_capability(account: &signer) {
        pragma verify = false;
    }

    spec find_delegation(addr: address): Option<u64> {
        aborts_if !exists<Delegations>(@core_resources);
    }

    spec schema ExistsCedraCoin {
        requires exists<coin::CoinInfo<CedraCoin>>(@cedra_framework);
    }

}
