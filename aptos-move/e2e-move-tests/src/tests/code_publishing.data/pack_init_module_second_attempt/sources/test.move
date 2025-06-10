module 0xcafe::test {
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::cedra_coin::CedraCoin;

    struct State has key {
        important_value: u64,
        coins: Coin<CedraCoin>,
    }

    fun init_module(s: &signer) {
        move_to(s, State {
            important_value: get_value(),
            coins: coin::zero<CedraCoin>(),
        })
    }

    fun get_value(): u64 {
        2
    }
}
