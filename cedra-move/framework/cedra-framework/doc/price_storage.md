
<a id="0x1_price_storage"></a>

# Module `0x1::price_storage`



-  [Struct `PriceInfoV2`](#0x1_price_storage_PriceInfoV2)
-  [Resource `PriceStorageV2`](#0x1_price_storage_PriceStorageV2)
-  [Struct `PriceUpdated`](#0x1_price_storage_PriceUpdated)
-  [Struct `PriceRemoved`](#0x1_price_storage_PriceRemoved)
-  [Struct `PriceInfo`](#0x1_price_storage_PriceInfo)
-  [Resource `PriceStorage`](#0x1_price_storage_PriceStorage)
-  [Resource `PriceTimestamps`](#0x1_price_storage_PriceTimestamps)
-  [Constants](#@Constants_0)
-  [Function `init_module`](#0x1_price_storage_init_module)
-  [Function `init_price_storage`](#0x1_price_storage_init_price_storage)
-  [Function `set_prices_v2`](#0x1_price_storage_set_prices_v2)
-  [Function `remove_price`](#0x1_price_storage_remove_price)
-  [Function `get_info`](#0x1_price_storage_get_info)
-  [Function `get`](#0x1_price_storage_get)
-  [Function `set_prices`](#0x1_price_storage_set_prices)
-  [Function `init_timestamps_storage`](#0x1_price_storage_init_timestamps_storage)
-  [Function `calculate_fa_fee`](#0x1_price_storage_calculate_fa_fee)
-  [Function `address_to_hex`](#0x1_price_storage_address_to_hex)
-  [Function `new_price_feed_id`](#0x1_price_storage_new_price_feed_id)
-  [Function `decode_oracle_price`](#0x1_price_storage_decode_oracle_price)
-  [Function `calculate_fa_fee_v2`](#0x1_price_storage_calculate_fa_fee_v2)


<pre><code><b>use</b> <a href="">0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c::i64</a>;
<b>use</b> <a href="">0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c::oracle</a>;
<b>use</b> <a href="">0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c::price</a>;
<b>use</b> <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../oracle-interface/../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/math128.md#0x1_math128">0x1::math128</a>;
<b>use</b> <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/string_utils.md#0x1_string_utils">0x1::string_utils</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/table.md#0x1_table">0x1::table</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_price_storage_PriceInfoV2"></a>

## Struct `PriceInfoV2`



<pre><code>#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceInfoV2">PriceInfoV2</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code><a href="">price</a>: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>decimals: u8</code>
</dt>
<dd>

</dd>
<dt>
<code><a href="timestamp.md#0x1_timestamp">timestamp</a>: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_storage_PriceStorageV2"></a>

## Resource `PriceStorageV2`



<pre><code>#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceStorageV2">PriceStorageV2</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>prices: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, <a href="price_storage.md#0x1_price_storage_PriceInfoV2">price_storage::PriceInfoV2</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_storage_PriceUpdated"></a>

## Struct `PriceUpdated`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceUpdated">PriceUpdated</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_storage_PriceRemoved"></a>

## Struct `PriceRemoved`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceRemoved">PriceRemoved</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_storage_PriceInfo"></a>

## Struct `PriceInfo`



<pre><code>#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceInfo">PriceInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code><a href="">price</a>: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>decimals: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_storage_PriceStorage"></a>

## Resource `PriceStorage`



<pre><code>#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceStorage">PriceStorage</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>prices: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, <a href="price_storage.md#0x1_price_storage_PriceInfo">price_storage::PriceInfo</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_storage_PriceTimestamps"></a>

## Resource `PriceTimestamps`



<pre><code>#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceTimestamps">PriceTimestamps</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>timestamps: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_price_storage_MAX_U64"></a>

MSB is used to indicate a gas payer tx


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a id="0x1_price_storage_EOUT_OF_GAS"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EOUT_OF_GAS">EOUT_OF_GAS</a>: u64 = 5;
</code></pre>



<a id="0x1_price_storage_CEDRA_FEED_ADDRESS"></a>

Cedra native feed identity for NewPriceIdentifier(address, symbol).


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_CEDRA_FEED_ADDRESS">CEDRA_FEED_ADDRESS</a>: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [48, 120, 49];
</code></pre>



<a id="0x1_price_storage_CEDRA_FEED_SYMBOL"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_CEDRA_FEED_SYMBOL">CEDRA_FEED_SYMBOL</a>: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [67, 101, 100, 114, 97];
</code></pre>



<a id="0x1_price_storage_DECIMALS_TOO_BIG"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_DECIMALS_TOO_BIG">DECIMALS_TOO_BIG</a>: u64 = 3;
</code></pre>



<a id="0x1_price_storage_EFA_FEE_OVERFLOW"></a>

Computed FA fee does not fit in u64.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EFA_FEE_OVERFLOW">EFA_FEE_OVERFLOW</a>: u64 = 10;
</code></pre>



<a id="0x1_price_storage_EPRICE_ALREADY_EXISTS"></a>

Price already exists in storage


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EPRICE_ALREADY_EXISTS">EPRICE_ALREADY_EXISTS</a>: u64 = 2;
</code></pre>



<a id="0x1_price_storage_EPRICE_CONFIDENCE_TOO_WIDE"></a>

Oracle confidence interval is too wide relative to price.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EPRICE_CONFIDENCE_TOO_WIDE">EPRICE_CONFIDENCE_TOO_WIDE</a>: u64 = 11;
</code></pre>



<a id="0x1_price_storage_EPRICE_NOT_FOUND"></a>

Price not founded in storage


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>: u64 = 1;
</code></pre>



<a id="0x1_price_storage_EPRICE_TIMESTAMP_IN_FUTURE"></a>

Oracle price timestamp is ahead of on-chain time.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EPRICE_TIMESTAMP_IN_FUTURE">EPRICE_TIMESTAMP_IN_FUTURE</a>: u64 = 9;
</code></pre>



<a id="0x1_price_storage_EPRICE_TOO_OLD"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EPRICE_TOO_OLD">EPRICE_TOO_OLD</a>: u64 = 6;
</code></pre>



<a id="0x1_price_storage_ESTORAGE_REFUND_EXCEEDS_FEE"></a>

Storage refund exceeds the gas fee amount.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_ESTORAGE_REFUND_EXCEEDS_FEE">ESTORAGE_REFUND_EXCEEDS_FEE</a>: u64 = 8;
</code></pre>



<a id="0x1_price_storage_ETIMESTAMPS_ALREADY_EXISTS"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_ETIMESTAMPS_ALREADY_EXISTS">ETIMESTAMPS_ALREADY_EXISTS</a>: u64 = 7;
</code></pre>



<a id="0x1_price_storage_FA_PRICE_IS_ZERO"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_FA_PRICE_IS_ZERO">FA_PRICE_IS_ZERO</a>: u64 = 4;
</code></pre>



<a id="0x1_price_storage_MAX_CONF_BPS"></a>

Max confidence / price in basis points (200 = 2%).


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_MAX_CONF_BPS">MAX_CONF_BPS</a>: u64 = 200;
</code></pre>



<a id="0x1_price_storage_MAX_PRICE_AGE"></a>

Max age of an oracle price relative to on-chain time (seconds).
60s is a common fee/trading default; tighten to ~30s if the publisher updates faster.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_MAX_PRICE_AGE">MAX_PRICE_AGE</a>: u64 = 60;
</code></pre>



<a id="0x1_price_storage_ORACLE_ADDRESS"></a>

On-chain oracle module target (documentation; bytecode links via @oracle named address).


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_ORACLE_ADDRESS">ORACLE_ADDRESS</a>: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [48, 120, 49, 48, 56, 99, 53, 54, 53, 49, 56, 57, 51, 54, 49, 55, 55, 100, 98, 100, 52, 51, 52, 98, 56, 50, 98, 53, 101, 48, 101, 101, 50, 56, 55, 97, 102, 102, 101, 98, 97, 53, 100, 55, 48, 50, 102, 97, 48, 100, 50, 55, 51, 52, 56, 101, 49, 54, 99, 55, 55, 98, 100, 97, 52, 99];
</code></pre>



<a id="0x1_price_storage_ORACLE_METHOD"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_ORACLE_METHOD">ORACLE_METHOD</a>: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [103, 101, 116, 95, 112, 114, 105, 99, 101, 95, 98, 121, 95, 102, 101, 101, 100, 95, 105, 100];
</code></pre>



<a id="0x1_price_storage_ORACLE_MODULE"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_ORACLE_MODULE">ORACLE_MODULE</a>: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [111, 114, 97, 99, 108, 101];
</code></pre>



<a id="0x1_price_storage_init_module"></a>

## Function `init_module`



<pre><code>#[deprecated]
<b>fun</b> <a href="price_storage.md#0x1_price_storage_init_module">init_module</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_init_module">init_module</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_price_storage_init_price_storage"></a>

## Function `init_price_storage`



<pre><code>#[deprecated]
<b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_price_storage">init_price_storage</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_price_storage">init_price_storage</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_price_storage_set_prices_v2"></a>

## Function `set_prices_v2`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices_v2">set_prices_v2</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfoV2">price_storage::PriceInfoV2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices_v2">set_prices_v2</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfoV2">PriceInfoV2</a>&gt;) {}
</code></pre>



</details>

<a id="0x1_price_storage_remove_price"></a>

## Function `remove_price`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_remove_price">remove_price</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_remove_price">remove_price</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _fa_address: String) {}
</code></pre>



</details>

<a id="0x1_price_storage_get_info"></a>

## Function `get_info`



<pre><code>#[deprecated]
<b>public</b>(<b>friend</b>) <b>fun</b> <a href="price_storage.md#0x1_price_storage_get_info">get_info</a>(_fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>): (u64, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="price_storage.md#0x1_price_storage_get_info">get_info</a>(_fa_address: String): (u64, u8) {
    (0, 0)
}
</code></pre>



</details>

<a id="0x1_price_storage_get"></a>

## Function `get`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_get">get</a>(_fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>): (u64, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_get">get</a>(_fa_address: String): (u64, u8) {
    (0, 0)
}
</code></pre>



</details>

<a id="0x1_price_storage_set_prices"></a>

## Function `set_prices`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices">set_prices</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfo">price_storage::PriceInfo</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices">set_prices</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfo">PriceInfo</a>&gt;) {}
</code></pre>



</details>

<a id="0x1_price_storage_init_timestamps_storage"></a>

## Function `init_timestamps_storage`



<pre><code>#[deprecated]
<b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_timestamps_storage">init_timestamps_storage</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_timestamps_storage">init_timestamps_storage</a>(_cedra_framework: &<a href="../../oracle-interface/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_price_storage_calculate_fa_fee"></a>

## Function `calculate_fa_fee`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee">calculate_fa_fee</a>(_gas_used: u64, _storage_fee_refunded: u64, _txn_gas_price: u64, _fa_address: <a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee">calculate_fa_fee</a>(
    _gas_used: u64,
    _storage_fee_refunded: u64,
    _txn_gas_price: u64,
    _fa_address: String,
): u64 {
    0
}
</code></pre>



</details>

<a id="0x1_price_storage_address_to_hex"></a>

## Function `address_to_hex`

Format address as <code>0x</code> + 64-char zero-padded lowercase hex.
Matches Go NewPriceIdentifier address strings (e.g. "0xc745ffa4...").
Note: to_string_with_canonical_addresses yields "@" + 64 hex with no "0x".


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_address_to_hex">address_to_hex</a>(addr: <b>address</b>): <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_address_to_hex">address_to_hex</a>(addr: <b>address</b>): <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> s = <a href="../../cedra-stdlib/doc/string_utils.md#0x1_string_utils_to_string_with_canonical_addresses">string_utils::to_string_with_canonical_addresses</a>(&addr);
    <b>let</b> hex = *<a href="../../oracle-interface/../move-stdlib/doc/string.md#0x1_string_bytes">string::bytes</a>(&s);
    // Strip leading `@` from "@&lt;64 hex&gt;"
    <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector_remove">vector::remove</a>(&<b>mut</b> hex, 0);
    <b>let</b> result = b"0x";
    <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> result, hex);
    result
}
</code></pre>



</details>

<a id="0x1_price_storage_new_price_feed_id"></a>

## Function `new_price_feed_id`

Matches Go NewPriceIdentifier: sha3_256(address_bytes || symbol_bytes) -> 32 bytes.


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_new_price_feed_id">new_price_feed_id</a>(address_bytes: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_new_price_feed_id">new_price_feed_id</a>(address_bytes: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> data = address_bytes;
    <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> data, symbol);
    <a href="../../oracle-interface/../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(data)
}
</code></pre>



</details>

<a id="0x1_price_storage_decode_oracle_price"></a>

## Function `decode_oracle_price`

Decode oracle Price into (price, decimals) used by the fee formula.


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_decode_oracle_price">decode_oracle_price</a>(p: &<a href="_Price">price::Price</a>, current_time: u64): (u64, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_decode_oracle_price">decode_oracle_price</a>(p: &Price, current_time: u64): (u64, u8) {
    <b>let</b> price_ts = <a href="_get_timestamp">price::get_timestamp</a>(p);
    <b>assert</b>!(price_ts &lt;= current_time, <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EPRICE_TIMESTAMP_IN_FUTURE">EPRICE_TIMESTAMP_IN_FUTURE</a>));
    <b>assert</b>!(
        current_time - price_ts &lt;= <a href="price_storage.md#0x1_price_storage_MAX_PRICE_AGE">MAX_PRICE_AGE</a>,
        <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EPRICE_TOO_OLD">EPRICE_TOO_OLD</a>)
    );

    <b>let</b> raw_price = <a href="_get_magnitude_if_positive">i64::get_magnitude_if_positive</a>(&<a href="_get_price">price::get_price</a>(p));
    <b>assert</b>!(raw_price &gt; 0, <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="price_storage.md#0x1_price_storage_FA_PRICE_IS_ZERO">FA_PRICE_IS_ZERO</a>));

    // conf and <a href="">price</a> share the same expo, so the ratio is scale-independent.
    <b>assert</b>!(
        (<a href="_get_conf">price::get_conf</a>(p) <b>as</b> u128) * 10000 &lt;= (raw_price <b>as</b> u128) * (<a href="price_storage.md#0x1_price_storage_MAX_CONF_BPS">MAX_CONF_BPS</a> <b>as</b> u128),
        <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EPRICE_CONFIDENCE_TOO_WIDE">EPRICE_CONFIDENCE_TOO_WIDE</a>)
    );

    <b>let</b> expo = <a href="_get_expo">price::get_expo</a>(p);
    <b>let</b> decimals = (<a href="_get_magnitude_if_negative">i64::get_magnitude_if_negative</a>(&expo) <b>as</b> u8);
    <b>assert</b>!(decimals &lt;= 18, <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_DECIMALS_TOO_BIG">DECIMALS_TOO_BIG</a>));

    (raw_price, decimals)
}
</code></pre>



</details>

<a id="0x1_price_storage_calculate_fa_fee_v2"></a>

## Function `calculate_fa_fee_v2`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee_v2">calculate_fa_fee_v2</a>(gas_used: u64, storage_fee_refunded: u64, txn_gas_price: u64, fa_address: <b>address</b>, symbol: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee_v2">calculate_fa_fee_v2</a>(
    gas_used: u64,
    storage_fee_refunded: u64,
    txn_gas_price: u64,
    fa_address: <b>address</b>,
    symbol: <a href="../../oracle-interface/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): u64 {
    // Keep <b>module</b> constants aligned <b>with</b> the linked @<a href="">oracle</a> call target.
    <b>assert</b>!(<a href="price_storage.md#0x1_price_storage_ORACLE_MODULE">ORACLE_MODULE</a> == b"<a href="">oracle</a>", <a href="price_storage.md#0x1_price_storage_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>);
    <b>assert</b>!(<a href="price_storage.md#0x1_price_storage_ORACLE_METHOD">ORACLE_METHOD</a> == b"get_price_by_feed_id", <a href="price_storage.md#0x1_price_storage_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>);
    <b>assert</b>!(<a href="price_storage.md#0x1_price_storage_ORACLE_ADDRESS">ORACLE_ADDRESS</a> == b"0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c", <a href="price_storage.md#0x1_price_storage_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>);

    <b>let</b> current_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();

    <b>assert</b>!(
        (txn_gas_price <b>as</b> u128) * (gas_used <b>as</b> u128) &lt;= <a href="price_storage.md#0x1_price_storage_MAX_U64">MAX_U64</a>,
        <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EOUT_OF_GAS">EOUT_OF_GAS</a>)
    );

    <b>let</b> transaction_fee_amount = txn_gas_price * gas_used;
    <b>assert</b>!(
        storage_fee_refunded &lt;= transaction_fee_amount,
        <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_ESTORAGE_REFUND_EXCEEDS_FEE">ESTORAGE_REFUND_EXCEEDS_FEE</a>)
    );
    <b>let</b> cedra_fee_amount = transaction_fee_amount - storage_fee_refunded;
    <b>if</b> (cedra_fee_amount == 0) {
        <b>return</b> 0
    };

    <b>let</b> fa_feed_id = <a href="price_storage.md#0x1_price_storage_new_price_feed_id">new_price_feed_id</a>(<a href="price_storage.md#0x1_price_storage_address_to_hex">address_to_hex</a>(fa_address), symbol);
    <b>let</b> fa_oracle_price = <a href="_get_price_by_feed_id">oracle::get_price_by_feed_id</a>(fa_feed_id);
    <b>let</b> (fa_price, fa_decimals) = <a href="price_storage.md#0x1_price_storage_decode_oracle_price">decode_oracle_price</a>(&fa_oracle_price, current_time);

    <b>let</b> cedra_feed_id = <a href="price_storage.md#0x1_price_storage_new_price_feed_id">new_price_feed_id</a>(<a href="price_storage.md#0x1_price_storage_CEDRA_FEED_ADDRESS">CEDRA_FEED_ADDRESS</a>, <a href="price_storage.md#0x1_price_storage_CEDRA_FEED_SYMBOL">CEDRA_FEED_SYMBOL</a>);
    <b>let</b> cedra_oracle_price = <a href="_get_price_by_feed_id">oracle::get_price_by_feed_id</a>(cedra_feed_id);
    <b>let</b> (cedra_price, cedra_decimals) = <a href="price_storage.md#0x1_price_storage_decode_oracle_price">decode_oracle_price</a>(&cedra_oracle_price, current_time);

    // fa_fee = (cedra_fee * cedra_price * 10^fa_decimals) / (fa_price * 10^cedra_decimals)
    // Use u128 mul_div (u256 intermediate) and reject results that do not fit in u64.
    <b>let</b> fa_fee_u128 = <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_mul_div">math128::mul_div</a>(
        <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_mul_div">math128::mul_div</a>(
            (cedra_fee_amount <b>as</b> u128),
            (cedra_price <b>as</b> u128),
            <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_pow">math128::pow</a>(10, (cedra_decimals <b>as</b> u128))
        ),
        <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_pow">math128::pow</a>(10, (fa_decimals <b>as</b> u128)),
        (fa_price <b>as</b> u128)
    );
    <b>assert</b>!(fa_fee_u128 &lt;= <a href="price_storage.md#0x1_price_storage_MAX_U64">MAX_U64</a>, <a href="../../oracle-interface/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EFA_FEE_OVERFLOW">EFA_FEE_OVERFLOW</a>));

    <b>let</b> fa_fee = (fa_fee_u128 <b>as</b> u64);
    // Charge at least 1 FA unit when net Cedra fee is positive but division truncates <b>to</b> 0.
    <b>if</b> (fa_fee == 0) {
        1
    } <b>else</b> {
        fa_fee
    }
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
