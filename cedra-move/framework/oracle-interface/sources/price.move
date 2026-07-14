/// Compile-time interface stub for the on-chain oracle Price type.
/// Layout must match the deployed oracle::price module.
module oracle::price {
    use oracle::i64::I64;

    struct Price has copy, drop, store {
        price: I64,
        conf: u64,
        expo: I64,
        timestamp: u64,
    }

    public fun get_price(_price: &Price): I64 {
        abort 0
    }

    public fun get_conf(_price: &Price): u64 {
        abort 0
    }

    public fun get_timestamp(_price: &Price): u64 {
        abort 0
    }

    public fun get_expo(_price: &Price): I64 {
        abort 0
    }
}
