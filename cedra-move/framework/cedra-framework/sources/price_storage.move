module cedra_framework::price_storage {
    use std::vector;
    use std::error;
    use std::string::{Self, String};
    use cedra_std::table::{Self, Table};
    use cedra_framework::system_addresses;
    use cedra_framework::event::emit;
    use cedra_std::math64;

    friend cedra_framework::transaction_validation;

    /// Price not founded in storage
    const EPRICE_NOT_FOUND: u64 = 1;
    /// Price already exists in storage
    const EPRICE_ALREADY_EXISTS: u64 = 2;
    const DECIMALS_TOO_BIG: u64 = 3;
    const FA_PRICE_IS_ZERO: u64 = 4;


    struct PriceInfo has copy, drop, store {
        fa_address: String,
        price: u64,
        decimals: u8
    }

    struct PriceStorage has key, store {
        prices: Table<String, PriceInfo>,
    }

    #[event]
    struct PriceUpdated has drop, store { fa_address: String }

    #[event]
    struct PriceRemoved has drop, store { fa_address: String }

    public entry fun init_price_storage(cedra_framework: &signer) {
        system_addresses::assert_cedra_framework(cedra_framework);

        assert!(
            !exists<PriceStorage>(@cedra_framework),
            EPRICE_ALREADY_EXISTS
        );

        move_to(
            cedra_framework,
            PriceStorage {
                prices: table::new<String, PriceInfo>(),
            }
        );
    }

    public fun set_prices(
        cedra_framework: &signer,
        prices: vector<PriceInfo>
    ) acquires PriceStorage {
        system_addresses::assert_cedra_framework(cedra_framework);
        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        let i = 0;
        let n = vector::length(&prices);
        while (i < n) {
            let price_info = *vector::borrow(&prices, i);
 
            table::upsert(
                &mut store.prices,
                price_info.fa_address,
                price_info
            );

            emit(PriceUpdated { fa_address: price_info.fa_address });

            i = i + 1;
        }
    }

    public fun remove_price(
        cedra_framework: &signer,
        fa_address: String
    ) acquires PriceStorage {
        system_addresses::assert_cedra_framework(cedra_framework);
        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        if (table::contains(&store.prices, fa_address)) {
            table::remove(&mut store.prices, fa_address);
            emit(PriceRemoved { fa_address });
        }
    }

    public(friend) fun get_info(fa_address: String): (u64, u8) 
    acquires PriceStorage {
        let store = borrow_global<PriceStorage>(@cedra_framework);

        assert!(
            table::contains(&store.prices, fa_address),
            EPRICE_NOT_FOUND
        );

        let price_info = table::borrow(&store.prices, fa_address);
        (price_info.price, price_info.decimals)
    }

        #[view]
        public fun get(fa_address: String): (u64, u8) 
    acquires PriceStorage {
        let store = borrow_global<PriceStorage>(@cedra_framework);


        assert!(
            table::contains(&store.prices, fa_address),
            EPRICE_NOT_FOUND
        );

        let price_info = table::borrow(&store.prices, fa_address);
        (price_info.price, price_info.decimals)
    }

    #[view]
    public fun calculate_fa_fee(
        cedra_fee_amount: u64,
        fa_address: String,
    ): u64 acquires PriceStorage {
        let store = borrow_global<PriceStorage>(@cedra_framework);

        // Get FA price and decimals
        assert!(table::contains(&store.prices, fa_address), EPRICE_NOT_FOUND);
        let fa_info = table::borrow(&store.prices, fa_address);
        let fa_price = fa_info.price;
        let fa_decimals = fa_info.decimals;
        assert!(fa_price > 0, error::invalid_argument(FA_PRICE_IS_ZERO));
        assert!(fa_decimals <= 18, error::out_of_range(DECIMALS_TOO_BIG));

        // Get Cedra price and decimals
        let cedra_address = string::utf8(b"0x1::cedra_coin::CedraCoin");
        assert!(table::contains(&store.prices, cedra_address), EPRICE_NOT_FOUND);
        let cedra_info = table::borrow(&store.prices, cedra_address);
        let cedra_price = cedra_info.price;
        let cedra_decimals = cedra_info.decimals;


        //todo: change location of description and leave here only minimal one
        // Calculate the equivalent fee amount in FA tokens based on Cedra fee amount
        // Formula: fa_fee = (cedra_fee * cedra_price * 10^fa_decimals) / (fa_price * 10^cedra_decimals)
        // Why we use mul_div in two steps:
        // 1. Direct multiplication could overflow: cedra_fee * cedra_price * 10^fa_decimals might exceed u64::MAX            // 2. mul_div uses u128 internally to prevent intermediate overflow
        // 3. We break the calculation into safe steps:
        //    Step 1: (cedra_fee * cedra_price) / 10^cedra_decimals
        //    Step 2: (step1_result * 10^fa_decimals) / fa_price 
        //
        // This is mathematically identical to the original formula but safe from overflow.

        // Example: Convert 100 Cedra tokens to FA tokens
        // cedra_fee_amount = 100 (100 Cedra tokens)
        // cedra_price = 2_000_000 (=$20.00 with 5 decimals: 20 * 10^5)
        // cedra_decimals = 5
        // fa_price = 50_000_000 (=$50.00 with 6 decimals: 50 * 10^6)  
        // fa_decimals = 6
        //
        // Step 1: (100 * 2,000,000) / 100,000 = 2,000
        // Step 2: (2,000 * 1,000,000) / 50,000,000 = 40 FA tokens
        //
        // Result: 100 Cedra ($20 each) = $2,000 = 40 FA ($50 each)
        let normalized_cedra_value = math64::mul_div(
            cedra_fee_amount * 1000,
            cedra_price,
            math64::pow(10, cedra_decimals as u64)
        );

        let fa_fee_amount = math64::mul_div(
            normalized_cedra_value,
            math64::pow(10, fa_decimals as u64),
            fa_price
        );

        fa_fee_amount
    }

}
