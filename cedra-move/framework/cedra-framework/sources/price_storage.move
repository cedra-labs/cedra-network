// This module provides an interface to store, add and remove prices of stablecoins from list.
module cedra_framework::price_storage {
    use std::signer;
    use std::vector;
    use cedra_framework::system_addresses;
        
    /// Price not founded in storage
    const EPRICE_NOT_FOUND: u64 = 1;

    struct PriceInfo has copy, drop, store {
        fa_address: vector<u8>,
        // scaled by decimals (e.g. real_price * 10^decimals)
        price: u64,
        decimals: u8
    }

    struct PriceStorage has key {
        prices: vector<PriceInfo>
    }

    public fun initialize(cedra_framework: &signer) {
        system_addresses::assert_cedra_framework(cedra_framework);
        if (!exists<PriceStorage>(@cedra_framework)) {
            move_to<PriceStorage>(
                cedra_framework,
                PriceStorage {
                    prices: vector::empty<PriceInfo>()
                }
            );
        }
    }

    public fun set_price(
        account: &signer, fa_address: vector<u8>, price: u64, decimals: u8
    ) acquires PriceStorage {
        let store = borrow_global_mut<PriceStorage>(signer::address_of(account));
        let len = vector::length(&store.prices);
        let i = 0;

        while (i < len) {
            let p_ref = vector::borrow_mut(&mut store.prices, i);

            if (p_ref.fa_address == fa_address) {
                p_ref.price = price;
                p_ref.decimals = decimals;
                return;
            };
            i = i + 1;
        };

        vector::push_back(
            &mut store.prices,
            PriceInfo { fa_address, price, decimals }
        );
    }

    public fun remove_price(account: &signer, fa_address: vector<u8>) acquires PriceStorage {
        let store = borrow_global_mut<PriceStorage>(signer::address_of(account));
        let len = vector::length(&store.prices);
        let i = 0;

        while (i < len) {
            let p_ref = vector::borrow(&store.prices, i);
            if (p_ref.fa_address == fa_address) {
                let last = vector::pop_back(&mut store.prices);
                if (i < vector::length(&store.prices)) {
                    let p_mut = vector::borrow_mut(&mut store.prices, i);
                    *p_mut = last;
                };
                return;
            };
            i = i + 1;
        };
        abort EPRICE_NOT_FOUND
    }

    public fun get_price(account: &signer, fa_address: vector<u8>): (u64, u8) acquires PriceStorage {
        let store = borrow_global<PriceStorage>(signer::address_of(account));
        let len = vector::length(&store.prices);
        let i = 0;

        while (i < len) {
            let p_ref = vector::borrow(&store.prices, i);
            if (p_ref.fa_address == fa_address) {
                return (p_ref.price, p_ref.decimals);
            };
            i = i + 1;
        };
        abort EPRICE_NOT_FOUND
    }
}
