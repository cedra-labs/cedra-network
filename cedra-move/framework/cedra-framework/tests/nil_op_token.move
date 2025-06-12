#[test_only]
module 0xcafe::nil_op_token {
    use cedra_framework::fungible_asset::{Self, FungibleAsset, TransferRef};
    use cedra_framework::dispatchable_fungible_asset;
    use cedra_framework::object::{ConstructorRef, Object};
    use cedra_framework::function_info;

    use std::option;
    use std::signer;
    use std::string;

    public fun initialize(account: &signer, constructor_ref: &ConstructorRef) {
        assert!(signer::address_of(account) == @0xcafe, 1);
        let withdraw = function_info::new_function_info(
            account,
            string::utf8(b"nil_op_token"),
            string::utf8(b"withdraw"),
        );

        dispatchable_fungible_asset::register_dispatch_functions(
            constructor_ref,
            option::some(withdraw),
            option::none(),
            option::none(),
        );
    }

    public fun withdraw<T: key>(
        store: Object<T>,
        _amount: u64,
        transfer_ref: &TransferRef,
    ): FungibleAsset {
        // Always return a one FA.
        fungible_asset::withdraw_with_ref(transfer_ref, store, 1)
    }
}
