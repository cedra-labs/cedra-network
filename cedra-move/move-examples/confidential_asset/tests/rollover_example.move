#[test_only]
module confidential_asset_example::rollover_example {
    use std::signer;
    use std::string::utf8;
    use cedra_std::debug::print;
    use cedra_framework::fungible_asset::Metadata;
    use cedra_framework::object::Object;

    use cedra_experimental::confidential_asset;
    use cedra_experimental::confidential_asset_tests;
    use cedra_experimental::ristretto255_twisted_elgamal as twisted_elgamal;

    fun rollover(bob: &signer, token: Object<Metadata>) {
        let bob_addr = signer::address_of(bob);

        // It's a test-only function, so we don't need to worry about the security of the keypair.
        let (bob_dk, bob_ek) = twisted_elgamal::generate_twisted_elgamal_keypair();

        let bob_ek = twisted_elgamal::pubkey_to_bytes(&bob_ek);

        let bob_amount = 100;

        confidential_asset::register(bob, token, bob_ek);
        confidential_asset::deposit(bob, token, bob_amount);

        print(&utf8(b"Bob's pending balance is NOT zero:"));
        print(&confidential_asset::pending_balance(bob_addr, token));

        print(&utf8(b"Bob's actual balance is zero:"));
        print(&confidential_asset::actual_balance(bob_addr, token));

        assert!(confidential_asset::verify_pending_balance(bob_addr, token, &bob_dk, bob_amount));
        assert!(confidential_asset::verify_actual_balance(bob_addr, token, &bob_dk, 0));

        // No explicit normalization is required, as the actual balance is already normalized.
        assert!(confidential_asset::is_normalized(bob_addr, token));

        confidential_asset::rollover_pending_balance(bob, token);

        print(&utf8(b"Bob's pending balance is zero:"));
        print(&confidential_asset::pending_balance(bob_addr, token));

        print(&utf8(b"Bob's actual balance is NOT zero:"));
        print(&confidential_asset::actual_balance(bob_addr, token));

        assert!(confidential_asset::verify_pending_balance(bob_addr, token, &bob_dk, 0));
        assert!(confidential_asset::verify_actual_balance(bob_addr, token, &bob_dk, (bob_amount as u128)));
    }

    #[test(
        confidential_asset = @cedra_experimental,
        cedra_fx = @cedra_framework,
        fa = @0xfa,
        bob = @0xb0
    )]
    fun rollover_example_test(
        confidential_asset: signer,
        cedra_fx: signer,
        fa: signer,
        bob: signer)
    {
        let token = confidential_asset_tests::set_up_for_confidential_asset_test(
            &confidential_asset,
            &cedra_fx,
            &fa,
            &bob,
            &bob,
            500,
            0
        );

        rollover(&bob, token);
    }
}
