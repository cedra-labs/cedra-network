module 0xcafe::test {
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::cedra_coin::CedraCoin;
    use std::signer::address_of;

    struct State has key {
        important_value: u64,
        coins: Coin<CedraCoin>,
    }

    fun init_module(s: &signer) {
        // Transfer away all the Cedra from s so there's nothing left to pay for gas.
        // This makes this init_module function fail for sure.
        let balance = coin::balance<CedraCoin>(address_of(s));
        let coins = coin::withdraw<CedraCoin>(s, balance);

        move_to(s, State {
            important_value: get_value(),
            coins,
        })
    }

    fun get_value(): u64 {
        1
    }
}
