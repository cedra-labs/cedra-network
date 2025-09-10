#[test_only]
module cedra_framework::cedra_coin_tests {
    use cedra_framework::cedra_coin;
    use cedra_framework::coin;
    use cedra_framework::fungible_asset::{Self, FungibleStore, Metadata};
    use cedra_framework::primary_fungible_store;
    use cedra_framework::object::{Self, Object};

    public fun mint_cedra_fa_to_for_test<T: key>(store: Object<T>, amount: u64) {
        fungible_asset::deposit(store, cedra_coin::mint_cedra_fa_for_test(amount));
    }

    public fun mint_cedra_fa_to_primary_fungible_store_for_test(
        owner: address,
        amount: u64,
    ) {
        primary_fungible_store::deposit(owner, cedra_coin::mint_cedra_fa_for_test(amount));
    }

    #[test(cedra_framework = @cedra_framework)]
    fun test_cedra_setup_and_mint(cedra_framework: &signer) {
        let (burn_cap, mint_cap) = cedra_coin::initialize_for_test(cedra_framework);
        let coin = coin::mint(100, &mint_cap);
        let fa = coin::coin_to_fungible_asset(coin);
        primary_fungible_store::deposit(@cedra_framework, fa);
        assert!(
            primary_fungible_store::balance(
                @cedra_framework,
                object::address_to_object<Metadata>(@cedra_fungible_asset)
            ) == 100,
            0
        );
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);
    }

    #[test]
    fun test_fa_helpers_for_test() {
        assert!(!object::object_exists<Metadata>(@cedra_fungible_asset), 0);
        cedra_coin::ensure_initialized_with_cedra_fa_metadata_for_test();
        assert!(object::object_exists<Metadata>(@cedra_fungible_asset), 0);
        mint_cedra_fa_to_primary_fungible_store_for_test(@cedra_framework, 100);
        let metadata = object::address_to_object<Metadata>(@cedra_fungible_asset);
        assert!(primary_fungible_store::balance(@cedra_framework, metadata) == 100, 0);
        let store_addr = primary_fungible_store::primary_store_address(@cedra_framework, metadata);
        mint_cedra_fa_to_for_test(object::address_to_object<FungibleStore>(store_addr), 100);
        assert!(primary_fungible_store::balance(@cedra_framework, metadata) == 200, 0);
    }
}
