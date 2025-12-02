// This module provides an interface to store, add and remove prices of stablecoins from list.
module cedra_framework::price_storage {
    use std::string::String;
    use cedra_std::table::{Self,Table};
    use cedra_framework::system_addresses;
    use cedra_framework::event::emit;

    friend cedra_framework::transaction_validation;

    /// Price not founded in storage
    const EPRICE_NOT_FOUND: u64 = 1;
    /// Price already exists in storage
    const EPRICE_ALREADY_EXISTS: u64 = 2;

    struct PriceInfo has copy, drop, store {
        fa_address: String,
        price: u64,
        decimals: u8
    }

    struct PriceStorage has key, store {
        prices: Table<String, PriceInfo>,
    }

    #[event]
    struct PriceUpdated has drop, store {
        fa_address: String,
    }

    #[event]
    struct PriceRemoved has drop, store {
        fa_address: String
    }

    public entry fun init_price_storage(cedra_framework: &signer) {
        system_addresses::assert_cedra_framework(cedra_framework);

        assert!(
            !exists<PriceStorage>(@cedra_framework),
            EPRICE_ALREADY_EXISTS
        );

        move_to<PriceStorage>(
            cedra_framework,
            PriceStorage {
            prices: table::new<String, PriceInfo>(),
            }
        );
    }

    public fun set_price(
        cedra_framework: &signer,
        price_info: PriceInfo
    ) acquires PriceStorage {
        system_addresses::assert_cedra_framework(cedra_framework);

        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        table::upsert(
            &mut store.prices,
            price_info.fa_address,
            price_info
        );

        emit(
            PriceUpdated {
                fa_address: price_info.fa_address
            }
        );
    }

    public fun remove_price(cedra_framework: &signer, fa_address: String) acquires PriceStorage {
        system_addresses::assert_cedra_framework(cedra_framework);
        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        table::remove(&mut store.prices, fa_address);

        emit(
            PriceRemoved { fa_address }
        );
    }

    public(friend) fun get_info(addr_str: String): (u64, u8) acquires PriceStorage {
        let store = borrow_global<PriceStorage>(@cedra_framework);

        if (!table::contains(&store.prices, addr_str)) {
            abort EPRICE_NOT_FOUND;
        };

        let price_info = table::borrow(&store.prices, addr_str);
        (price_info.price, price_info.decimals)
    }

  }
