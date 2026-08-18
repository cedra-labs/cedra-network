
<a id="0x1_price_storage"></a>

# Module `0x1::price_storage`



-  [Struct `OracleQuote`](#0x1_price_storage_OracleQuote)
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
-  [Function `new_oracle_quote`](#0x1_price_storage_new_oracle_quote)
-  [Function `decode_oracle_quote`](#0x1_price_storage_decode_oracle_quote)
-  [Function `calculate_fa_fee_v2`](#0x1_price_storage_calculate_fa_fee_v2)
-  [Function `calculate_fa_fee_from_quotes`](#0x1_price_storage_calculate_fa_fee_from_quotes)


<pre><code><b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/math128.md#0x1_math128">0x1::math128</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/string_utils.md#0x1_string_utils">0x1::string_utils</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/table.md#0x1_table">0x1::table</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_price_storage_OracleQuote"></a>

## Struct `OracleQuote`

BCS layout of an oracle Price unpacked by the VM.


<pre><code><b>struct</b> <a href="price_storage.md#0x1_price_storage_OracleQuote">OracleQuote</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>price_negative: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>price_magnitude: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>conf: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>expo_negative: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>expo_magnitude: u64</code>
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

<a id="0x1_price_storage_PriceInfoV2"></a>

## Struct `PriceInfoV2`



<pre><code>#[deprecated]
<b>struct</b> <a href="price_storage.md#0x1_price_storage_PriceInfoV2">PriceInfoV2</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>price: u64</code>
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
<code>prices: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, <a href="price_storage.md#0x1_price_storage_PriceInfoV2">price_storage::PriceInfoV2</a>&gt;</code>
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
<code>fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
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
<code>fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
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
<code>fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>price: u64</code>
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
<code>prices: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, <a href="price_storage.md#0x1_price_storage_PriceInfo">price_storage::PriceInfo</a>&gt;</code>
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
<code>timestamps: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, u64&gt;</code>
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


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_CEDRA_FEED_ADDRESS">CEDRA_FEED_ADDRESS</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [48, 120, 49];
</code></pre>



<a id="0x1_price_storage_CEDRA_FEED_SYMBOL"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_CEDRA_FEED_SYMBOL">CEDRA_FEED_SYMBOL</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [67, 101, 100, 114, 97];
</code></pre>



<a id="0x1_price_storage_DECIMALS_TOO_BIG"></a>



<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_DECIMALS_TOO_BIG">DECIMALS_TOO_BIG</a>: u64 = 3;
</code></pre>



<a id="0x1_price_storage_EFA_FEE_OVERFLOW"></a>

Computed FA fee does not fit in u64.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EFA_FEE_OVERFLOW">EFA_FEE_OVERFLOW</a>: u64 = 10;
</code></pre>



<a id="0x1_price_storage_EORACLE_FETCH_REQUIRES_VM"></a>

Oracle fetch is performed by the VM from OracleConfig; this entry is view metadata only.


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_EORACLE_FETCH_REQUIRES_VM">EORACLE_FETCH_REQUIRES_VM</a>: u64 = 12;
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


<pre><code><b>const</b> <a href="price_storage.md#0x1_price_storage_MAX_PRICE_AGE">MAX_PRICE_AGE</a>: u64 = 60;
</code></pre>



<a id="0x1_price_storage_init_module"></a>

## Function `init_module`



<pre><code>#[deprecated]
<b>fun</b> <a href="price_storage.md#0x1_price_storage_init_module">init_module</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_init_module">init_module</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_price_storage_init_price_storage"></a>

## Function `init_price_storage`



<pre><code>#[deprecated]
<b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_price_storage">init_price_storage</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_price_storage">init_price_storage</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_price_storage_set_prices_v2"></a>

## Function `set_prices_v2`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices_v2">set_prices_v2</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfoV2">price_storage::PriceInfoV2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices_v2">set_prices_v2</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfoV2">PriceInfoV2</a>&gt;) {}
</code></pre>



</details>

<a id="0x1_price_storage_remove_price"></a>

## Function `remove_price`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_remove_price">remove_price</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_remove_price">remove_price</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _fa_address: String) {}
</code></pre>



</details>

<a id="0x1_price_storage_get_info"></a>

## Function `get_info`



<pre><code>#[deprecated]
<b>public</b>(<b>friend</b>) <b>fun</b> <a href="price_storage.md#0x1_price_storage_get_info">get_info</a>(_fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>): (u64, u8)
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
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_get">get</a>(_fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>): (u64, u8)
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
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices">set_prices</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfo">price_storage::PriceInfo</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_set_prices">set_prices</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _prices: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="price_storage.md#0x1_price_storage_PriceInfo">PriceInfo</a>&gt;) {}
</code></pre>



</details>

<a id="0x1_price_storage_init_timestamps_storage"></a>

## Function `init_timestamps_storage`



<pre><code>#[deprecated]
<b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_timestamps_storage">init_timestamps_storage</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="price_storage.md#0x1_price_storage_init_timestamps_storage">init_timestamps_storage</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_price_storage_calculate_fa_fee"></a>

## Function `calculate_fa_fee`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee">calculate_fa_fee</a>(_gas_used: u64, _storage_fee_refunded: u64, _txn_gas_price: u64, _fa_address: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>): u64
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


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_address_to_hex">address_to_hex</a>(addr: <b>address</b>): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_address_to_hex">address_to_hex</a>(addr: <b>address</b>): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> s = <a href="../../cedra-stdlib/doc/string_utils.md#0x1_string_utils_to_string_with_canonical_addresses">string_utils::to_string_with_canonical_addresses</a>(&addr);
    <b>let</b> hex = *<a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_bytes">string::bytes</a>(&s);
    // Strip leading `@` from "@&lt;64 hex&gt;"
    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_remove">vector::remove</a>(&<b>mut</b> hex, 0);
    <b>let</b> result = b"0x";
    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> result, hex);
    result
}
</code></pre>



</details>

<a id="0x1_price_storage_new_price_feed_id"></a>

## Function `new_price_feed_id`

Matches Go NewPriceIdentifier: sha3_256(address_bytes || symbol_bytes) -> 32 bytes.


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_new_price_feed_id">new_price_feed_id</a>(address_bytes: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_new_price_feed_id">new_price_feed_id</a>(address_bytes: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> data = address_bytes;
    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> data, symbol);
    <a href="../../cedra-stdlib/../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(data)
}
</code></pre>



</details>

<a id="0x1_price_storage_new_oracle_quote"></a>

## Function `new_oracle_quote`



<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_new_oracle_quote">new_oracle_quote</a>(price_negative: bool, price_magnitude: u64, conf: u64, expo_negative: bool, expo_magnitude: u64, <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64): <a href="price_storage.md#0x1_price_storage_OracleQuote">price_storage::OracleQuote</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_new_oracle_quote">new_oracle_quote</a>(
    price_negative: bool,
    price_magnitude: u64,
    conf: u64,
    expo_negative: bool,
    expo_magnitude: u64,
    <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64,
): <a href="price_storage.md#0x1_price_storage_OracleQuote">OracleQuote</a> {
    <a href="price_storage.md#0x1_price_storage_OracleQuote">OracleQuote</a> {
        price_negative,
        price_magnitude,
        conf,
        expo_negative,
        expo_magnitude,
        <a href="timestamp.md#0x1_timestamp">timestamp</a>,
    }
}
</code></pre>



</details>

<a id="0x1_price_storage_decode_oracle_quote"></a>

## Function `decode_oracle_quote`

Decode an oracle quote into (price, decimals) used by the fee formula.


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_decode_oracle_quote">decode_oracle_quote</a>(q: &<a href="price_storage.md#0x1_price_storage_OracleQuote">price_storage::OracleQuote</a>, current_time: u64): (u64, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="price_storage.md#0x1_price_storage_decode_oracle_quote">decode_oracle_quote</a>(q: &<a href="price_storage.md#0x1_price_storage_OracleQuote">OracleQuote</a>, current_time: u64): (u64, u8) {
    <b>assert</b>!(q.<a href="timestamp.md#0x1_timestamp">timestamp</a> &lt;= current_time, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EPRICE_TIMESTAMP_IN_FUTURE">EPRICE_TIMESTAMP_IN_FUTURE</a>));
    <b>assert</b>!(
        current_time - q.<a href="timestamp.md#0x1_timestamp">timestamp</a> &lt;= <a href="price_storage.md#0x1_price_storage_MAX_PRICE_AGE">MAX_PRICE_AGE</a>,
        <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EPRICE_TOO_OLD">EPRICE_TOO_OLD</a>)
    );

    <b>assert</b>!(!q.price_negative && q.price_magnitude &gt; 0, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="price_storage.md#0x1_price_storage_FA_PRICE_IS_ZERO">FA_PRICE_IS_ZERO</a>));
    <b>let</b> raw_price = q.price_magnitude;

    <b>assert</b>!(
        (q.conf <b>as</b> u128) * 10000 &lt;= (raw_price <b>as</b> u128) * (<a href="price_storage.md#0x1_price_storage_MAX_CONF_BPS">MAX_CONF_BPS</a> <b>as</b> u128),
        <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EPRICE_CONFIDENCE_TOO_WIDE">EPRICE_CONFIDENCE_TOO_WIDE</a>)
    );

    <b>assert</b>!(q.expo_negative, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_DECIMALS_TOO_BIG">DECIMALS_TOO_BIG</a>));
    <b>let</b> decimals = (q.expo_magnitude <b>as</b> u8);
    <b>assert</b>!(decimals &lt;= 18, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_DECIMALS_TOO_BIG">DECIMALS_TOO_BIG</a>));

    (raw_price, decimals)
}
</code></pre>



</details>

<a id="0x1_price_storage_calculate_fa_fee_v2"></a>

## Function `calculate_fa_fee_v2`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee_v2">calculate_fa_fee_v2</a>(_gas_used: u64, _storage_fee_refunded: u64, _txn_gas_price: u64, _fa_address: <b>address</b>, _symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee_v2">calculate_fa_fee_v2</a>(
    _gas_used: u64,
    _storage_fee_refunded: u64,
    _txn_gas_price: u64,
    _fa_address: <b>address</b>,
    _symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): u64 {
    <b>abort</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="price_storage.md#0x1_price_storage_EORACLE_FETCH_REQUIRES_VM">EORACLE_FETCH_REQUIRES_VM</a>)
}
</code></pre>



</details>

<a id="0x1_price_storage_calculate_fa_fee_from_quotes"></a>

## Function `calculate_fa_fee_from_quotes`



<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee_from_quotes">calculate_fa_fee_from_quotes</a>(gas_used: u64, storage_fee_refunded: u64, txn_gas_price: u64, fa: <a href="price_storage.md#0x1_price_storage_OracleQuote">price_storage::OracleQuote</a>, cedra: <a href="price_storage.md#0x1_price_storage_OracleQuote">price_storage::OracleQuote</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="price_storage.md#0x1_price_storage_calculate_fa_fee_from_quotes">calculate_fa_fee_from_quotes</a>(
    gas_used: u64,
    storage_fee_refunded: u64,
    txn_gas_price: u64,
    fa: <a href="price_storage.md#0x1_price_storage_OracleQuote">OracleQuote</a>,
    cedra: <a href="price_storage.md#0x1_price_storage_OracleQuote">OracleQuote</a>,
): u64 {
    <b>let</b> current_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();

    <b>assert</b>!(
        (txn_gas_price <b>as</b> u128) * (gas_used <b>as</b> u128) &lt;= <a href="price_storage.md#0x1_price_storage_MAX_U64">MAX_U64</a>,
        <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EOUT_OF_GAS">EOUT_OF_GAS</a>)
    );

    <b>let</b> transaction_fee_amount = txn_gas_price * gas_used;
    <b>assert</b>!(
        storage_fee_refunded &lt;= transaction_fee_amount,
        <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_ESTORAGE_REFUND_EXCEEDS_FEE">ESTORAGE_REFUND_EXCEEDS_FEE</a>)
    );
    <b>let</b> cedra_fee_amount = transaction_fee_amount - storage_fee_refunded;
    <b>if</b> (cedra_fee_amount == 0) {
        <b>return</b> 0
    };

    <b>let</b> (fa_price, fa_decimals) = <a href="price_storage.md#0x1_price_storage_decode_oracle_quote">decode_oracle_quote</a>(&fa, current_time);
    <b>let</b> (cedra_price, cedra_decimals) = <a href="price_storage.md#0x1_price_storage_decode_oracle_quote">decode_oracle_quote</a>(&cedra, current_time);

    <b>let</b> fa_fee_u128 = <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_mul_div">math128::mul_div</a>(
        <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_mul_div">math128::mul_div</a>(
            (cedra_fee_amount <b>as</b> u128),
            (cedra_price <b>as</b> u128),
            <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_pow">math128::pow</a>(10, (cedra_decimals <b>as</b> u128))
        ),
        <a href="../../cedra-stdlib/doc/math128.md#0x1_math128_pow">math128::pow</a>(10, (fa_decimals <b>as</b> u128)),
        (fa_price <b>as</b> u128)
    );
    <b>assert</b>!(fa_fee_u128 &lt;= <a href="price_storage.md#0x1_price_storage_MAX_U64">MAX_U64</a>, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="price_storage.md#0x1_price_storage_EFA_FEE_OVERFLOW">EFA_FEE_OVERFLOW</a>));

    <b>let</b> fa_fee = (fa_fee_u128 <b>as</b> u64);
    <b>if</b> (fa_fee == 0) {
        1
    } <b>else</b> {
        fa_fee
    }
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
