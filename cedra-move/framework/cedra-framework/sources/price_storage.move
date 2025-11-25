// This module provides an interface to store, add and remove prices of stablecoins from list.
module cedra_framework::price_storage {
    use std::vector;
    use cedra_std::table::{Self,Table};
    use cedra_framework::system_addresses;
    use cedra_framework::event::emit;

    friend cedra_framework::transaction_validation;

    /// Price not founded in storage
    const EPRICE_NOT_FOUND: u64 = 1;
    /// Price already exists in storage
    const EPRICE_ALREADY_EXISTS: u64 = 2;

    struct PriceInfo has copy, drop, store {
        fa_address: vector<u8>,
        price: u64,
        decimals: u8
    }

    struct PriceStorage has key, store {
        prices: Table<vector<u8>, PriceInfo>,
        version: u64
    }

    #[event]
    /// When `PriceStorage` is updated, this event is sent to resync the Oracle consensus state in all validators.
    struct PriceUpdated has drop, store {
        version: u64,
        prices: vector<PriceInfo>
    }

    #[event]
    struct PriceRemoved has drop, store {
        version: u64,
        fa_address: vector<u8>
    }

    public entry fun init_price_storage(cedra_framework: &signer) {
        system_addresses::assert_cedra_framework(cedra_framework);

        if (!exists<PriceStorage>(@cedra_framework)) {
            move_to<PriceStorage>(
                cedra_framework,
                PriceStorage {
                    prices: table::new<vector<u8>, PriceInfo>(),
                    version: 0
                }
            );
        }
    }

    public fun set_prices(
        cedra_framework: &signer, new_prices: vector<PriceInfo>
    ) acquires PriceStorage {
        system_addresses::assert_cedra_framework(cedra_framework);

        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        let i = 0;
        let len = vector::length(&new_prices);

        while (i < len) {
            let price_info = vector::borrow(&new_prices, i);
            table::upsert(&mut store.prices, price_info.fa_address, *price_info);
            i = i + 1;
        };

        store.version = store.version + 1;

        emit(
            PriceUpdated { version: store.version, prices: new_prices }
        );
    }

    public fun remove_price(cedra_framework: &signer, fa_address: vector<u8>) acquires PriceStorage {
        system_addresses::assert_cedra_framework(cedra_framework);
        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        table::remove(&mut store.prices, fa_address);
        store.version = store.version + 1;

        emit(
            PriceRemoved { version: store.version, fa_address }
        );
    }

    public fun get_version(): u64 acquires PriceStorage {
        borrow_global<PriceStorage>(@cedra_framework).version
    }

    public fun get_price(fa_address: vector<u8>): PriceInfo acquires PriceStorage {
        let store = borrow_global_mut<PriceStorage>(@cedra_framework);

        if (!table::contains(&store.prices, fa_address)) {
            abort EPRICE_NOT_FOUND;
        };

        *table::borrow(&store.prices, fa_address)
    }
}
