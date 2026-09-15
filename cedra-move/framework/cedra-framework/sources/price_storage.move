module cedra_framework::price_storage {
    use std::vector;
    use std::error;
    use std::hash;
    use std::string::{Self, String};
    use cedra_std::table::Table;
    use cedra_std::string_utils;
    use cedra_std::math128;
    use cedra_framework::timestamp;

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
    /// Storage refund exceeds the gas fee amount.
    const ESTORAGE_REFUND_EXCEEDS_FEE: u64 = 8;
    /// Oracle price timestamp is ahead of on-chain time.
    const EPRICE_TIMESTAMP_IN_FUTURE: u64 = 9;
    /// Computed FA fee does not fit in u64.
    const EFA_FEE_OVERFLOW: u64 = 10;
    /// Oracle confidence interval is too wide relative to price.
    const EPRICE_CONFIDENCE_TOO_WIDE: u64 = 11;
    /// Oracle fetch is performed by the VM from OracleConfig; this entry is view metadata only.
    const EORACLE_FETCH_REQUIRES_VM: u64 = 12;
    /// MSB is used to indicate a gas payer tx
    const MAX_U64: u128 = 18446744073709551615;
    /// Max age of an oracle price relative to on-chain time (seconds).
    const MAX_PRICE_AGE: u64 = 60;
    /// Max confidence / price in basis points (200 = 2%).
    const MAX_CONF_BPS: u64 = 200;

    /// Cedra native feed identity for NewPriceIdentifier(address, symbol).
    const CEDRA_FEED_ADDRESS: vector<u8> = b"0x1";
    const CEDRA_FEED_SYMBOL: vector<u8> = b"Cedra";

    /// BCS layout of an oracle Price unpacked by the VM.
    struct OracleQuote has copy, drop {
        price_negative: bool,
        price_magnitude: u64,
        conf: u64,
        expo_negative: bool,
        expo_magnitude: u64,
        timestamp: u64,
    }

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

    public fun new_oracle_quote(
        price_negative: bool,
        price_magnitude: u64,
        conf: u64,
        expo_negative: bool,
        expo_magnitude: u64,
        timestamp: u64,
    ): OracleQuote {
        OracleQuote {
            price_negative,
            price_magnitude,
            conf,
            expo_negative,
            expo_magnitude,
            timestamp,
        }
    }

    /// Decode an oracle quote into (price, decimals) used by the fee formula.
    fun decode_oracle_quote(q: &OracleQuote, current_time: u64): (u64, u8) {
        assert!(q.timestamp <= current_time, error::out_of_range(EPRICE_TIMESTAMP_IN_FUTURE));
        assert!(
            current_time - q.timestamp <= MAX_PRICE_AGE,
            error::out_of_range(EPRICE_TOO_OLD)
        );

        assert!(!q.price_negative && q.price_magnitude > 0, error::invalid_argument(FA_PRICE_IS_ZERO));
        let raw_price = q.price_magnitude;

        assert!(
            (q.conf as u128) * 10000 <= (raw_price as u128) * (MAX_CONF_BPS as u128),
            error::out_of_range(EPRICE_CONFIDENCE_TOO_WIDE)
        );

        assert!(q.expo_negative, error::out_of_range(DECIMALS_TOO_BIG));
        let decimals = (q.expo_magnitude as u8);
        assert!(decimals <= 18, error::out_of_range(DECIMALS_TOO_BIG));

        (raw_price, decimals)
    }

    /// VM-backed view: the node fetches prices from `OracleConfig.addr` and evaluates the fee.
    /// Direct Move execution of this function aborts; use `calculate_fa_fee_from_quotes`.
    #[view]
    public fun calculate_fa_fee_v2(
        _gas_used: u64,
        _storage_fee_refunded: u64,
        _txn_gas_price: u64,
        _fa_address: address,
        _symbol: vector<u8>
    ): u64 {
        abort error::invalid_state(EORACLE_FETCH_REQUIRES_VM)
    }

    public fun calculate_fa_fee_from_quotes(
        gas_used: u64,
        storage_fee_refunded: u64,
        txn_gas_price: u64,
        fa: OracleQuote,
        cedra: OracleQuote,
    ): u64 {
        let current_time = timestamp::now_seconds();

        assert!(
            (txn_gas_price as u128) * (gas_used as u128) <= MAX_U64,
            error::out_of_range(EOUT_OF_GAS)
        );

        let transaction_fee_amount = txn_gas_price * gas_used;
        assert!(
            storage_fee_refunded <= transaction_fee_amount,
            error::out_of_range(ESTORAGE_REFUND_EXCEEDS_FEE)
        );
        let cedra_fee_amount = transaction_fee_amount - storage_fee_refunded;
        if (cedra_fee_amount == 0) {
            return 0
        };

        let (fa_price, fa_decimals) = decode_oracle_quote(&fa, current_time);
        let (cedra_price, cedra_decimals) = decode_oracle_quote(&cedra, current_time);

        let fa_fee_u128 = math128::mul_div(
            math128::mul_div(
                (cedra_fee_amount as u128),
                (cedra_price as u128),
                math128::pow(10, (cedra_decimals as u128))
            ),
            math128::pow(10, (fa_decimals as u128)),
            (fa_price as u128)
        );
        assert!(fa_fee_u128 <= MAX_U64, error::out_of_range(EFA_FEE_OVERFLOW));

        let fa_fee = (fa_fee_u128 as u64);
        if (fa_fee == 0) {
            1
        } else {
            fa_fee
        }
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
