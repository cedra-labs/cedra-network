module cedra_framework::price_storage {
    use std::vector;
    use std::error;
    use std::hash;
    use std::string::{Self, String};
    use cedra_std::table::Table;
    use cedra_std::string_utils;
    use cedra_framework::timestamp;
    use cedra_std::math64;
    use oracle::oracle;
    use oracle::price::{Self, Price};
    use oracle::i64;

    friend cedra_framework::transaction_validation;

    /// Price not founded in storage
    const EPRICE_NOT_FOUND: u64 = 1;
    /// Price already exists in storage
    const EPRICE_ALREADY_EXISTS: u64 = 2;
    const DECIMALS_TOO_BIG: u64 = 3;
    const FA_PRICE_IS_ZERO: u64 = 4;
    const EOUT_OF_GAS: u64 = 5;
    const EPRICE_TOO_OLD: u64 = 6;
    const ETIMESTAMPS_ALREADY_EXISTS: u64 = 7;
    /// MSB is used to indicate a gas payer tx
    const MAX_U64: u128 = 18446744073709551615;
    const MAX_PRICE_AGE: u64 = 60;

    /// On-chain oracle module target (documentation; bytecode links via @oracle named address).
    const ORACLE_ADDRESS: vector<u8> = b"0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c";
    const ORACLE_MODULE: vector<u8> = b"oracle";
    const ORACLE_METHOD: vector<u8> = b"get_price_by_feed_id";

    /// Cedra native feed identity for NewPriceIdentifier(address, symbol).
    const CEDRA_FEED_ADDRESS: vector<u8> = b"0x1";
    const CEDRA_FEED_SYMBOL: vector<u8> = b"Cedra";

    #[deprecated]
    struct PriceInfoV2 has copy, drop, store {
        fa_address: String,
        price: u64,
        decimals: u8,
        timestamp: u64,
    }

    #[deprecated]
    struct PriceStorageV2 has key, store {
        prices: Table<String, PriceInfoV2>,
    }

    #[event]
    #[deprecated]
    struct PriceUpdated has drop, store { fa_address: String }

    #[event]
    #[deprecated]
    struct PriceRemoved has drop, store { fa_address: String }

    #[deprecated]
    struct PriceInfo has copy, drop, store {
        fa_address: String,
        price: u64,
        decimals: u8
    }

    #[deprecated]
    struct PriceStorage has key, store {
        prices: Table<String, PriceInfo>,
    }

    #[deprecated]
    struct PriceTimestamps has key, store {
        timestamps: Table<String, u64>,
    }

    // localnet init_module
    #[deprecated]
    fun init_module(_cedra_framework: &signer) {}

    #[deprecated]
    public entry fun init_price_storage(_cedra_framework: &signer) {}

    #[deprecated]
    public fun set_prices_v2(_cedra_framework: &signer, _prices: vector<PriceInfoV2>) {}

    #[deprecated]
    public fun remove_price(_cedra_framework: &signer, _fa_address: String) {}

    #[deprecated]
    public(friend) fun get_info(_fa_address: String): (u64, u8) {
        (0, 0)
    }

    #[deprecated]
    public fun get(_fa_address: String): (u64, u8) {
        (0, 0)
    }

    #[deprecated]
    public fun set_prices(_cedra_framework: &signer, _prices: vector<PriceInfo>) {}

    #[deprecated]
    public entry fun init_timestamps_storage(_cedra_framework: &signer) {}

    #[deprecated]
    public fun calculate_fa_fee(
        _gas_used: u64,
        _storage_fee_refunded: u64,
        _txn_gas_price: u64,
        _fa_address: String,
    ): u64 {
        0
    }

    /// Format address as `0x` + 64-char zero-padded lowercase hex.
    /// Matches Go NewPriceIdentifier address strings (e.g. "0xc745ffa4...").
    /// Note: to_string_with_canonical_addresses yields "@" + 64 hex with no "0x".
    fun address_to_hex(addr: address): vector<u8> {
        let s = string_utils::to_string_with_canonical_addresses(&addr);
        let hex = *string::bytes(&s);
        // Strip leading `@` from "@<64 hex>"
        vector::remove(&mut hex, 0);
        let result = b"0x";
        vector::append(&mut result, hex);
        result
    }

    /// Matches Go NewPriceIdentifier: sha3_256(address_bytes || symbol_bytes) -> 32 bytes.
    fun new_price_feed_id(address_bytes: vector<u8>, symbol: vector<u8>): vector<u8> {
        let data = address_bytes;
        vector::append(&mut data, symbol);
        hash::sha3_256(data)
    }

    /// Decode oracle Price into (price, decimals) used by the fee formula.
    fun decode_oracle_price(p: &Price, current_time: u64): (u64, u8) {
        assert!(
            current_time - price::get_timestamp(p) <= MAX_PRICE_AGE,
            error::out_of_range(EPRICE_TOO_OLD)
        );

        let raw_price = i64::get_magnitude_if_positive(&price::get_price(p));
        assert!(raw_price > 0, error::invalid_argument(FA_PRICE_IS_ZERO));

        let expo = price::get_expo(p);
        let decimals = (i64::get_magnitude_if_negative(&expo) as u8);
        assert!(decimals <= 18, error::out_of_range(DECIMALS_TOO_BIG));

        (raw_price, decimals)
    }

    #[view]
    public fun calculate_fa_fee_v2(
        gas_used: u64,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        fa_address: address,
        symbol: vector<u8>
    ): u64 {
        // Keep module constants aligned with the linked @oracle call target.
        assert!(ORACLE_MODULE == b"oracle", EPRICE_NOT_FOUND);
        assert!(ORACLE_METHOD == b"get_price_by_feed_id", EPRICE_NOT_FOUND);
        assert!(ORACLE_ADDRESS == b"0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c", EPRICE_NOT_FOUND);

        let current_time = timestamp::now_seconds();

        assert!(
            (txn_gas_price as u128) * (gas_used as u128) <= MAX_U64,
            error::out_of_range(EOUT_OF_GAS)
        );

        let transaction_fee_amount = txn_gas_price * gas_used;
        let cedra_fee_amount = transaction_fee_amount - storage_fee_refunded;

        let fa_feed_id = new_price_feed_id(address_to_hex(fa_address), symbol);
        let fa_oracle_price = oracle::get_price_by_feed_id(fa_feed_id);
        let (fa_price, fa_decimals) = decode_oracle_price(&fa_oracle_price, current_time);

        let cedra_feed_id = new_price_feed_id(CEDRA_FEED_ADDRESS, CEDRA_FEED_SYMBOL);
        let cedra_oracle_price = oracle::get_price_by_feed_id(cedra_feed_id);
        let (cedra_price, cedra_decimals) = decode_oracle_price(&cedra_oracle_price, current_time);

        // fa_fee = (cedra_fee * cedra_price * 10^fa_decimals) / (fa_price * 10^cedra_decimals)
        let normalized_cedra_value = math64::mul_div(
            cedra_fee_amount,
            cedra_price,
            math64::pow(10, (cedra_decimals as u64))
        );

        math64::mul_div(
            normalized_cedra_value,
            math64::pow(10, (fa_decimals as u64)),
            fa_price
        )
    }

    #[test]
    fun test_new_price_feed_id_cedra() {
        let feed_id = new_price_feed_id(CEDRA_FEED_ADDRESS, CEDRA_FEED_SYMBOL);
        assert!(
            feed_id == x"ab5cfdf4863d0717a8ebe5608f54f508e73b620c4d3855fbbacf8a9dd32c0d13",
            1
        );
    }

    #[test]
    fun test_address_to_hex_and_fa_feed_id() {
        let addr = @0xc745ffa4f97fa9739fae0cb173996f70bb8e4b0310fa781ccca2f7dc13f7db06;
        let hex = address_to_hex(addr);
        assert!(
            hex == b"0xc745ffa4f97fa9739fae0cb173996f70bb8e4b0310fa781ccca2f7dc13f7db06",
            1
        );
        let feed_id = new_price_feed_id(hex, b"USDCT");
        assert!(
            feed_id == x"9d4a9a43175e027d694d9fb4943feeeaa07dc0a5424bc51012de1bda83e43059",
            2
        );
    }
}
