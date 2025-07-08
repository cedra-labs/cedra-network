#[test_only]
module marketplace::test_utils {
    use std::signer;
    use std::string;
    use std::vector;

    use cedra_framework::account;
    use cedra_framework::cedra_coin::{Self, CedraCoin};
    use cedra_framework::coin;
    use cedra_framework::object::{Self, Object};
    use cedra_framework::timestamp;

    use cedra_token::token as tokenv1;
    use cedra_token_objects::token::Token;
    use cedra_token_objects::cedra_token;
    use cedra_token_objects::collection::Collection;

    use marketplace::fee_schedule::{Self, FeeSchedule};

    public inline fun setup(
        cedra_framework: &signer,
        marketplace: &signer,
        seller: &signer,
        purchaser: &signer,
    ): (address, address, address) {
        timestamp::set_time_has_started_for_testing(cedra_framework);
        let (burn_cap, mint_cap) = cedra_coin::initialize_for_test(cedra_framework);

        let marketplace_addr = signer::address_of(marketplace);
        account::create_account_for_test(marketplace_addr);
        coin::register<CedraCoin>(marketplace);

        let seller_addr = signer::address_of(seller);
        account::create_account_for_test(seller_addr);
        coin::register<CedraCoin>(seller);

        let purchaser_addr = signer::address_of(purchaser);
        account::create_account_for_test(purchaser_addr);
        coin::register<CedraCoin>(purchaser);

        let coins = coin::mint(10000, &mint_cap);
        coin::deposit(seller_addr, coins);
        let coins = coin::mint(10000, &mint_cap);
        coin::deposit(purchaser_addr, coins);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);

        (marketplace_addr, seller_addr, purchaser_addr)
    }

    public fun fee_schedule(seller: &signer): Object<FeeSchedule> {
        fee_schedule::init(
            seller,
            signer::address_of(seller),
            2,
            1,
            100,
            1,
        )
    }

    public inline fun increment_timestamp(seconds: u64) {
        timestamp::update_global_time_for_test(timestamp::now_microseconds() + (seconds * 1000000));
    }

    public fun mint_tokenv2_with_collection(seller: &signer): (Object<Collection>, Object<Token>) {
        let collection_name = string::utf8(b"collection_name");

        let collection_object = cedra_token::create_collection_object(
            seller,
            string::utf8(b"collection description"),
            2,
            collection_name,
            string::utf8(b"collection uri"),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            1,
            100,
        );

        let cedra_token = cedra_token::mint_token_object(
            seller,
            collection_name,
            string::utf8(b"description"),
            string::utf8(b"token_name"),
            string::utf8(b"uri"),
            vector::empty(),
            vector::empty(),
            vector::empty(),
        );
        (object::convert(collection_object), object::convert(cedra_token))
    }

    public fun mint_tokenv2_with_collection_royalty(
        seller: &signer,
        royalty_numerator: u64,
        royalty_denominator: u64
    ): (Object<Collection>, Object<Token>) {
        let collection_name = string::utf8(b"collection_name");

        let collection_object = cedra_token::create_collection_object(
            seller,
            string::utf8(b"collection description"),
            2,
            collection_name,
            string::utf8(b"collection uri"),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            royalty_numerator,
            royalty_denominator,
        );

        let cedra_token = cedra_token::mint_token_object(
            seller,
            collection_name,
            string::utf8(b"description"),
            string::utf8(b"token_name"),
            string::utf8(b"uri"),
            vector::empty(),
            vector::empty(),
            vector::empty(),
        );
        (object::convert(collection_object), object::convert(cedra_token))
    }

    public fun mint_tokenv2(seller: &signer): Object<Token> {
        let (_collection, token) = mint_tokenv2_with_collection(seller);
        token
    }

    public fun mint_tokenv2_additional(seller: &signer): Object<Token> {
        let collection_name = string::utf8(b"collection_name");

        let cedra_token = cedra_token::mint_token_object(
            seller,
            collection_name,
            string::utf8(b"description"),
            string::utf8(b"token_name_2"),
            string::utf8(b"uri"),
            vector::empty(),
            vector::empty(),
            vector::empty(),
        );
        object::convert(cedra_token)
    }

    public fun mint_tokenv1(seller: &signer): tokenv1::TokenId {
        let collection_name = string::utf8(b"collection_name");
        let token_name = string::utf8(b"token_name");

        tokenv1::create_collection(
            seller,
            collection_name,
            string::utf8(b"Collection: Hello, World"),
            string::utf8(b"https://cedra.dev"),
            2,
            vector[true, true, true],
        );

        tokenv1::create_token_script(
            seller,
            collection_name,
            token_name,
            string::utf8(b"Hello, Token"),
            1,
            1,
            string::utf8(b"https://cedra.dev"),
            signer::address_of(seller),
            100,
            1,
            vector[true, true, true, true, true],
            vector::empty(),
            vector::empty(),
            vector::empty(),
        );

        tokenv1::create_token_id_raw(
            signer::address_of(seller),
            collection_name,
            token_name,
            0,
        )
    }

    public fun mint_tokenv1_additional(seller: &signer): tokenv1::TokenId {
        let collection_name = string::utf8(b"collection_name");
        let token_name = string::utf8(b"token_name_2");
        tokenv1::create_token_script(
            seller,
            collection_name,
            token_name,
            string::utf8(b"Hello, Token"),
            1,
            1,
            string::utf8(b"https://cedra.dev"),
            signer::address_of(seller),
            100,
            1,
            vector[true, true, true, true, true],
            vector::empty(),
            vector::empty(),
            vector::empty(),
        );

        tokenv1::create_token_id_raw(
            signer::address_of(seller),
            collection_name,
            token_name,
            0,
        )
    }

    public fun mint_tokenv1_additional_royalty(
        seller: &signer,
        royalty_numerator: u64,
        royalty_denominator: u64
    ): tokenv1::TokenId {
        let collection_name = string::utf8(b"collection_name");
        let token_name = string::utf8(b"token_name_2");
        tokenv1::create_token_script(
            seller,
            collection_name,
            token_name,
            string::utf8(b"Hello, Token"),
            1,
            1,
            string::utf8(b"https://cedra.dev"),
            signer::address_of(seller),
            royalty_denominator,
            royalty_numerator,
            vector[true, true, true, true, true],
            vector::empty(),
            vector::empty(),
            vector::empty(),
        );

        tokenv1::create_token_id_raw(
            signer::address_of(seller),
            collection_name,
            token_name,
            0,
        )
    }
}
