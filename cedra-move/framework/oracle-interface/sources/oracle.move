/// Compile-time interface stub for the on-chain oracle module.
/// Bytecode linked against this package resolves to the live module at @oracle.
module oracle::oracle {
    use oracle::price::Price;

    public fun get_price_by_feed_id(_feed_id: vector<u8>): Price {
        abort 0
    }
}
