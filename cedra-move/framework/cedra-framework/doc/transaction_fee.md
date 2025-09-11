
<a id="0x1_transaction_fee"></a>

# Module `0x1::transaction_fee`



-  [Resource `CedraCoinCapabilities`](#0x1_transaction_fee_CedraCoinCapabilities)
-  [Resource `CedraFABurnCapabilities`](#0x1_transaction_fee_CedraFABurnCapabilities)
-  [Resource `CedraCoinMintCapability`](#0x1_transaction_fee_CedraCoinMintCapability)
-  [Struct `FeeStatement`](#0x1_transaction_fee_FeeStatement)
-  [Struct `CustomFeeStatement`](#0x1_transaction_fee_CustomFeeStatement)
-  [Resource `CollectedFeesPerBlock`](#0x1_transaction_fee_CollectedFeesPerBlock)
-  [Constants](#@Constants_0)
-  [Function `burn_fee`](#0x1_transaction_fee_burn_fee)
-  [Function `burn_fee_v2`](#0x1_transaction_fee_burn_fee_v2)
-  [Function `mint_and_refund`](#0x1_transaction_fee_mint_and_refund)
-  [Function `store_cedra_coin_burn_cap`](#0x1_transaction_fee_store_cedra_coin_burn_cap)
-  [Function `convert_to_cedra_fa_burn_ref`](#0x1_transaction_fee_convert_to_cedra_fa_burn_ref)
-  [Function `store_cedra_coin_mint_cap`](#0x1_transaction_fee_store_cedra_coin_mint_cap)
-  [Function `emit_fee_statement`](#0x1_transaction_fee_emit_fee_statement)
-  [Function `emit_custom_fee_statement`](#0x1_transaction_fee_emit_custom_fee_statement)
-  [Function `get_metadata`](#0x1_transaction_fee_get_metadata)
-  [Function `fa_address`](#0x1_transaction_fee_fa_address)
-  [Function `metadata`](#0x1_transaction_fee_metadata)
-  [Function `get_balance`](#0x1_transaction_fee_get_balance)
-  [Function `initialize_fee_collection_and_distribution`](#0x1_transaction_fee_initialize_fee_collection_and_distribution)
-  [Function `upgrade_burn_percentage`](#0x1_transaction_fee_upgrade_burn_percentage)
-  [Function `initialize_storage_refund`](#0x1_transaction_fee_initialize_storage_refund)
-  [Specification](#@Specification_1)
    -  [High-level Requirements](#high-level-req)
    -  [Module-level Specification](#module-level-spec)
    -  [Resource `CollectedFeesPerBlock`](#@Specification_1_CollectedFeesPerBlock)
    -  [Function `burn_fee`](#@Specification_1_burn_fee)
    -  [Function `mint_and_refund`](#@Specification_1_mint_and_refund)
    -  [Function `store_cedra_coin_burn_cap`](#@Specification_1_store_cedra_coin_burn_cap)
    -  [Function `store_cedra_coin_mint_cap`](#@Specification_1_store_cedra_coin_mint_cap)
    -  [Function `emit_fee_statement`](#@Specification_1_emit_fee_statement)
    -  [Function `initialize_fee_collection_and_distribution`](#@Specification_1_initialize_fee_collection_and_distribution)
    -  [Function `initialize_storage_refund`](#@Specification_1_initialize_storage_refund)


<pre><code><b>use</b> <a href="cedra_account.md#0x1_cedra_account">0x1::cedra_account</a>;
<b>use</b> <a href="cedra_coin.md#0x1_cedra_coin">0x1::cedra_coin</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features">0x1::features</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stablecoin.md#0x1_stablecoin">0x1::stablecoin</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
<b>use</b> <a href="whitelist.md#0x1_whitelist">0x1::whitelist</a>;
</code></pre>



<a id="0x1_transaction_fee_CedraCoinCapabilities"></a>

## Resource `CedraCoinCapabilities`

Stores burn capability to burn the gas fees.


<pre><code><b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>burn_cap: <a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_transaction_fee_CedraFABurnCapabilities"></a>

## Resource `CedraFABurnCapabilities`

Stores burn capability to burn the gas fees.


<pre><code><b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>burn_ref: <a href="fungible_asset.md#0x1_fungible_asset_BurnRef">fungible_asset::BurnRef</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_transaction_fee_CedraCoinMintCapability"></a>

## Resource `CedraCoinMintCapability`

Stores mint capability to mint the refunds.


<pre><code><b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_cap: <a href="coin.md#0x1_coin_MintCapability">coin::MintCapability</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_transaction_fee_FeeStatement"></a>

## Struct `FeeStatement`

Breakdown of fee charge and refund for a transaction.
The structure is:

- Net charge or refund (not in the statement)
- total charge: total_charge_gas_units, matches <code>gas_used</code> in the on-chain <code>TransactionInfo</code>.
This is the sum of the sub-items below. Notice that there's potential precision loss when
the conversion between internal and external gas units and between native token and gas
units, so it's possible that the numbers don't add up exactly. -- This number is the final
charge, while the break down is merely informational.
- gas charge for execution (CPU time): <code>execution_gas_units</code>
- gas charge for IO (storage random access): <code>io_gas_units</code>
- storage fee charge (storage space): <code>storage_fee_octas</code>, to be included in
<code>total_charge_gas_unit</code>, this number is converted to gas units according to the user
specified <code>gas_unit_price</code> on the transaction.
- storage deletion refund: <code>storage_fee_refund_octas</code>, this is not included in <code>gas_used</code> or
<code>total_charge_gas_units</code>, the net charge / refund is calculated by
<code>total_charge_gas_units</code> * <code>gas_unit_price</code> - <code>storage_fee_refund_octas</code>.

This is meant to emitted as a module event.


<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_FeeStatement">FeeStatement</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total_charge_gas_units: u64</code>
</dt>
<dd>
 Total gas charge.
</dd>
<dt>
<code>execution_gas_units: u64</code>
</dt>
<dd>
 Execution gas charge.
</dd>
<dt>
<code>io_gas_units: u64</code>
</dt>
<dd>
 IO gas charge.
</dd>
<dt>
<code>storage_fee_octas: u64</code>
</dt>
<dd>
 Storage fee charge.
</dd>
<dt>
<code>storage_fee_refund_octas: u64</code>
</dt>
<dd>
 Storage fee refund.
</dd>
</dl>


</details>

<a id="0x1_transaction_fee_CustomFeeStatement"></a>

## Struct `CustomFeeStatement`

Breakdown of fee charge and refund for a transaction.
The structure is:

- Net charge or refund (not in the statement)
- total charge: total_charge_gas_units, matches <code>gas_used</code> in the on-chain <code>TransactionInfo</code>.
This is the sum of the sub-items below. Notice that there's potential precision loss when
the conversion between internal and external gas units and between native token and gas
units, so it's possible that the numbers don't add up exactly. -- This number is the final
charge, while the break down is merely informational.
- gas charge for execution (CPU time): <code>execution_gas_units</code>
- gas charge for IO (storage random access): <code>io_gas_units</code>
- storage fee charge (storage space): <code>storage_fee_octas</code>, to be included in
<code>total_charge_gas_unit</code>, this number is converted to gas units according to the user
specified <code>gas_unit_price</code> on the transaction.
- storage deletion refund: <code>storage_fee_refund_octas</code>, this is not included in <code>gas_used</code> or
<code>total_charge_gas_units</code>, the net charge / refund is calculated by
<code>total_charge_gas_units</code> * <code>gas_unit_price</code> - <code>storage_fee_refund_octas</code>.

This is meant to emitted as a module event.


<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CustomFeeStatement">CustomFeeStatement</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total_charge_gas_units: u64</code>
</dt>
<dd>
 Total gas charge.
</dd>
<dt>
<code>execution_gas_units: u64</code>
</dt>
<dd>
 Execution gas charge.
</dd>
<dt>
<code>io_gas_units: u64</code>
</dt>
<dd>
 IO gas charge.
</dd>
<dt>
<code>storage_fee_octas: u64</code>
</dt>
<dd>
 Storage fee charge.
</dd>
<dt>
<code>storage_fee_refund_octas: u64</code>
</dt>
<dd>
 Storage fee refund.
</dd>
</dl>


</details>

<a id="0x1_transaction_fee_CollectedFeesPerBlock"></a>

## Resource `CollectedFeesPerBlock`

DEPRECATED: Stores information about the block proposer and the amount of fees
collected when executing the block.


<pre><code>#[deprecated]
<b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: <a href="coin.md#0x1_coin_AggregatableCoin">coin::AggregatableCoin</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>burn_percentage: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_transaction_fee_EINSUFFICIENT_BALANCE"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>: u64 = 9;
</code></pre>



<a id="0x1_transaction_fee_EUNAUTHORIZED"></a>

Caller is not authorized to make this call


<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EUNAUTHORIZED">EUNAUTHORIZED</a>: u64 = 10;
</code></pre>



<a id="0x1_transaction_fee_EALREADY_COLLECTING_FEES"></a>

Gas fees are already being collected and the struct holding
information about collected amounts is already published.


<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EALREADY_COLLECTING_FEES">EALREADY_COLLECTING_FEES</a>: u64 = 1;
</code></pre>



<a id="0x1_transaction_fee_EASSET_EXISTS"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EASSET_EXISTS">EASSET_EXISTS</a>: u64 = 7;
</code></pre>



<a id="0x1_transaction_fee_EFA_GAS_CHARGING_NOT_ENABLED"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EFA_GAS_CHARGING_NOT_ENABLED">EFA_GAS_CHARGING_NOT_ENABLED</a>: u64 = 5;
</code></pre>



<a id="0x1_transaction_fee_EINVALID_BURN_PERCENTAGE"></a>

The burn percentage is out of range [0, 100].


<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EINVALID_BURN_PERCENTAGE">EINVALID_BURN_PERCENTAGE</a>: u64 = 3;
</code></pre>



<a id="0x1_transaction_fee_ENOT_OWNER"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ENOT_OWNER">ENOT_OWNER</a>: u64 = 6;
</code></pre>



<a id="0x1_transaction_fee_ENO_LONGER_SUPPORTED"></a>

No longer supported.


<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ENO_LONGER_SUPPORTED">ENO_LONGER_SUPPORTED</a>: u64 = 4;
</code></pre>



<a id="0x1_transaction_fee_burn_fee"></a>

## Function `burn_fee`

Burn transaction fees in epilogue.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee">burn_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee">burn_fee</a>(
    <a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64
) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a>, <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a>&gt;(@cedra_framework)) {
        <b>let</b> burn_ref =
            &<b>borrow_global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a>&gt;(@cedra_framework).burn_ref;
        <a href="cedra_account.md#0x1_cedra_account_burn_from_fungible_store_for_gas">cedra_account::burn_from_fungible_store_for_gas</a>(burn_ref, <a href="account.md#0x1_account">account</a>, fee);
    } <b>else</b> {
        <b>let</b> burn_cap = &<b>borrow_global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a>&gt;(@cedra_framework).burn_cap;
        <b>if</b> (<a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features_operations_default_to_fa_cedra_store_enabled">features::operations_default_to_fa_cedra_store_enabled</a>()) {
            <b>let</b> (burn_ref, burn_receipt) = <a href="coin.md#0x1_coin_get_paired_burn_ref">coin::get_paired_burn_ref</a>(burn_cap);
            <a href="cedra_account.md#0x1_cedra_account_burn_from_fungible_store_for_gas">cedra_account::burn_from_fungible_store_for_gas</a>(&burn_ref, <a href="account.md#0x1_account">account</a>, fee);
            <a href="coin.md#0x1_coin_return_paired_burn_ref">coin::return_paired_burn_ref</a>(burn_ref, burn_receipt);
        } <b>else</b> {
            <a href="coin.md#0x1_coin_burn_from_for_gas">coin::burn_from_for_gas</a>&lt;CedraCoin&gt;(<a href="account.md#0x1_account">account</a>, fee, burn_cap);
        };
    };
}
</code></pre>



</details>

<a id="0x1_transaction_fee_burn_fee_v2"></a>

## Function `burn_fee_v2`

Burn custom transaction fees in epilogue.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee_v2">burn_fee_v2</a>(from_addr: <b>address</b>, creator_addr: <b>address</b>, module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, fee: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee_v2">burn_fee_v2</a>(
    from_addr: <b>address</b>,
    creator_addr: <b>address</b>,
    module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    fee: u64
) {
    // 1000 - fee_v2 feature not enabled
    <b>assert</b>!(<a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features_fee_v2_enabled">features::fee_v2_enabled</a>(), 1000);

    // 1001 - <a href="whitelist.md#0x1_whitelist">whitelist</a> registry missing
    <b>assert</b>!(<a href="whitelist.md#0x1_whitelist_has_registry">whitelist::has_registry</a>(@admin), 1001);

    // 1002 - asset not registered in <a href="whitelist.md#0x1_whitelist">whitelist</a>
    <b>assert</b>!(<a href="whitelist.md#0x1_whitelist_asset_exists">whitelist::asset_exists</a>(creator_addr, module_name, symbol), 1002);

    // 1003 - insufficient FA balance
    <b>assert</b>!(<a href="transaction_fee.md#0x1_transaction_fee_get_balance">get_balance</a>(creator_addr, from_addr, symbol) &gt;= fee, 1003);

    // 1004 - admin not in authorized callers
    <b>assert</b>!(
        <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&<a href="stablecoin.md#0x1_stablecoin_authorized_callers">stablecoin::authorized_callers</a>(creator_addr, symbol), &@admin),
        1004
    );

    <a href="stablecoin.md#0x1_stablecoin_authorized_transfer">stablecoin::authorized_transfer</a>(
            creator_addr,
            @admin,
            from_addr,
            @admin,
            symbol,
            100
        );
       }
</code></pre>



</details>

<a id="0x1_transaction_fee_mint_and_refund"></a>

## Function `mint_and_refund`

Mint refund in epilogue.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_mint_and_refund">mint_and_refund</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, refund: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_mint_and_refund">mint_and_refund</a>(
    <a href="account.md#0x1_account">account</a>: <b>address</b>, refund: u64
) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a> {
    <b>let</b> mint_cap = &<b>borrow_global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a>&gt;(@cedra_framework).mint_cap;
    <b>let</b> refund_coin = <a href="coin.md#0x1_coin_mint">coin::mint</a>(refund, mint_cap);
    <a href="coin.md#0x1_coin_deposit_for_gas_fee">coin::deposit_for_gas_fee</a>(<a href="account.md#0x1_account">account</a>, refund_coin);
}
</code></pre>



</details>

<a id="0x1_transaction_fee_store_cedra_coin_burn_cap"></a>

## Function `store_cedra_coin_burn_cap`

Only called during genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_cedra_coin_burn_cap">store_cedra_coin_burn_cap</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, burn_cap: <a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_cedra_coin_burn_cap">store_cedra_coin_burn_cap</a>(
    cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, burn_cap: BurnCapability&lt;CedraCoin&gt;
) {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(cedra_framework);

    <b>if</b> (<a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features_operations_default_to_fa_cedra_store_enabled">features::operations_default_to_fa_cedra_store_enabled</a>()) {
        <b>let</b> burn_ref = <a href="coin.md#0x1_coin_convert_and_take_paired_burn_ref">coin::convert_and_take_paired_burn_ref</a>(burn_cap);
        <b>move_to</b>(cedra_framework, <a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a> { burn_ref });
    } <b>else</b> {
        <b>move_to</b>(cedra_framework, <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a> { burn_cap })
    }
}
</code></pre>



</details>

<a id="0x1_transaction_fee_convert_to_cedra_fa_burn_ref"></a>

## Function `convert_to_cedra_fa_burn_ref`



<pre><code><b>public</b> entry <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_convert_to_cedra_fa_burn_ref">convert_to_cedra_fa_burn_ref</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_convert_to_cedra_fa_burn_ref">convert_to_cedra_fa_burn_ref</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a> {
    <b>assert</b>!(<a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features_operations_default_to_fa_cedra_store_enabled">features::operations_default_to_fa_cedra_store_enabled</a>(), <a href="transaction_fee.md#0x1_transaction_fee_EFA_GAS_CHARGING_NOT_ENABLED">EFA_GAS_CHARGING_NOT_ENABLED</a>);
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(cedra_framework);
    <b>let</b> <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a> { burn_cap } =
        <b>move_from</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a>&gt;(<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(cedra_framework));
    <b>let</b> burn_ref = <a href="coin.md#0x1_coin_convert_and_take_paired_burn_ref">coin::convert_and_take_paired_burn_ref</a>(burn_cap);
    <b>move_to</b>(cedra_framework, <a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a> { burn_ref });
}
</code></pre>



</details>

<a id="0x1_transaction_fee_store_cedra_coin_mint_cap"></a>

## Function `store_cedra_coin_mint_cap`

Only called during genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_cedra_coin_mint_cap">store_cedra_coin_mint_cap</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, mint_cap: <a href="coin.md#0x1_coin_MintCapability">coin::MintCapability</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_cedra_coin_mint_cap">store_cedra_coin_mint_cap</a>(
    cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, mint_cap: MintCapability&lt;CedraCoin&gt;
) {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(cedra_framework);
    <b>move_to</b>(cedra_framework, <a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a> { mint_cap })
}
</code></pre>



</details>

<a id="0x1_transaction_fee_emit_fee_statement"></a>

## Function `emit_fee_statement`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_emit_fee_statement">emit_fee_statement</a>(fee_statement: <a href="transaction_fee.md#0x1_transaction_fee_FeeStatement">transaction_fee::FeeStatement</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_emit_fee_statement">emit_fee_statement</a>(fee_statement: <a href="transaction_fee.md#0x1_transaction_fee_FeeStatement">FeeStatement</a>) {
    <a href="event.md#0x1_event_emit">event::emit</a>(fee_statement)
}
</code></pre>



</details>

<a id="0x1_transaction_fee_emit_custom_fee_statement"></a>

## Function `emit_custom_fee_statement`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_emit_custom_fee_statement">emit_custom_fee_statement</a>(custom_fee_statement: <a href="transaction_fee.md#0x1_transaction_fee_CustomFeeStatement">transaction_fee::CustomFeeStatement</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_emit_custom_fee_statement">emit_custom_fee_statement</a>(
    custom_fee_statement: <a href="transaction_fee.md#0x1_transaction_fee_CustomFeeStatement">CustomFeeStatement</a>
) {
    <a href="event.md#0x1_event_emit">event::emit</a>(custom_fee_statement)
}
</code></pre>



</details>

<a id="0x1_transaction_fee_get_metadata"></a>

## Function `get_metadata`

Return the address of the managed fungible asset that's created when this module is deployed.


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_get_metadata">get_metadata</a>(creator: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_get_metadata">get_metadata</a>(creator: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): Object&lt;Metadata&gt; {
    <b>let</b> asset_address = <a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&creator, symbol);
    <a href="object.md#0x1_object_address_to_object">object::address_to_object</a>&lt;Metadata&gt;(asset_address)
}
</code></pre>



</details>

<a id="0x1_transaction_fee_fa_address"></a>

## Function `fa_address`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_fa_address">fa_address</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_fa_address">fa_address</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <b>address</b> {
    <a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&owner, symbol)
}
</code></pre>



</details>

<a id="0x1_transaction_fee_metadata"></a>

## Function `metadata`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_metadata">metadata</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_metadata">metadata</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): Object&lt;Metadata&gt; {
    <a href="object.md#0x1_object_address_to_object">object::address_to_object</a>(<a href="transaction_fee.md#0x1_transaction_fee_fa_address">fa_address</a>(owner, symbol))
}
</code></pre>



</details>

<a id="0x1_transaction_fee_get_balance"></a>

## Function `get_balance`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_get_balance">get_balance</a>(admin: <b>address</b>, <a href="account.md#0x1_account">account</a>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_get_balance">get_balance</a>(
    admin: <b>address</b>, <a href="account.md#0x1_account">account</a>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): u64 {
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_balance">primary_fungible_store::balance</a>(<a href="account.md#0x1_account">account</a>, <a href="transaction_fee.md#0x1_transaction_fee_metadata">metadata</a>(admin, symbol))
}
</code></pre>



</details>

<a id="0x1_transaction_fee_initialize_fee_collection_and_distribution"></a>

## Function `initialize_fee_collection_and_distribution`

DEPRECATED


<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">initialize_fee_collection_and_distribution</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _burn_percentage: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">initialize_fee_collection_and_distribution</a>(
    _cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _burn_percentage: u8
) {
    <b>abort</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_not_implemented">error::not_implemented</a>(<a href="transaction_fee.md#0x1_transaction_fee_ENO_LONGER_SUPPORTED">ENO_LONGER_SUPPORTED</a>)
}
</code></pre>



</details>

<a id="0x1_transaction_fee_upgrade_burn_percentage"></a>

## Function `upgrade_burn_percentage`

DEPRECATED


<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_upgrade_burn_percentage">upgrade_burn_percentage</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _new_burn_percentage: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_upgrade_burn_percentage">upgrade_burn_percentage</a>(
    _cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _new_burn_percentage: u8
) {
    <b>abort</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_not_implemented">error::not_implemented</a>(<a href="transaction_fee.md#0x1_transaction_fee_ENO_LONGER_SUPPORTED">ENO_LONGER_SUPPORTED</a>)
}
</code></pre>



</details>

<a id="0x1_transaction_fee_initialize_storage_refund"></a>

## Function `initialize_storage_refund`



<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_storage_refund">initialize_storage_refund</a>(_: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_storage_refund">initialize_storage_refund</a>(_: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>abort</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_not_implemented">error::not_implemented</a>(<a href="transaction_fee.md#0x1_transaction_fee_ENO_LONGER_SUPPORTED">ENO_LONGER_SUPPORTED</a>)
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification




<a id="high-level-req"></a>

### High-level Requirements

<table>
<tr>
<th>No.</th><th>Requirement</th><th>Criticality</th><th>Implementation</th><th>Enforcement</th>
</tr>

<tr>
<td>1</td>
<td>Given the blockchain is in an operating state, it guarantees that the Cedra framework signer may burn Cedra coins.</td>
<td>Critical</td>
<td>The CedraCoinCapabilities structure is defined in this module and it stores burn capability to burn the gas fees.</td>
<td>Formally Verified via <a href="#high-level-req-1">module</a>.</td>
</tr>

<tr>
<td>2</td>
<td>The initialization function may only be called once.</td>
<td>Medium</td>
<td>The initialize_fee_collection_and_distribution function ensures CollectedFeesPerBlock does not already exist.</td>
<td>Formally verified via <a href="#high-level-req-2">initialize_fee_collection_and_distribution</a>.</td>
</tr>

<tr>
<td>3</td>
<td>Only the admin address is authorized to call the initialization function.</td>
<td>Critical</td>
<td>The initialize_fee_collection_and_distribution function ensures only the Cedra framework address calls it.</td>
<td>Formally verified via <a href="#high-level-req-3">initialize_fee_collection_and_distribution</a>.</td>
</tr>

<tr>
<td>4</td>
<td>The percentage of the burnt collected fee is always a value from 0 to 100.</td>
<td>Medium</td>
<td>During the initialization of CollectedFeesPerBlock in Initialize_fee_collection_and_distribution, and while upgrading burn percentage, it asserts that burn_percentage is within the specified limits.</td>
<td>Formally verified via <a href="#high-level-req-4">CollectedFeesPerBlock</a>.</td>
</tr>

<tr>
<td>5</td>
<td>Prior to upgrading the burn percentage, it must process all the fees collected up to that point.</td>
<td>Critical</td>
<td>The upgrade_burn_percentage function ensures process_collected_fees function is called before updating the burn percentage.</td>
<td>Formally verified in <a href="#high-level-req-5">ProcessCollectedFeesRequiresAndEnsures</a>.</td>
</tr>

<tr>
<td>6</td>
<td>The presence of the resource, indicating collected fees per block under the Cedra framework account, is a prerequisite for the successful execution of the following functionalities: Upgrading burn percentage. Registering a block proposer. Processing collected fees.</td>
<td>Low</td>
<td>The functions: upgrade_burn_percentage, register_proposer_for_fee_collection, and process_collected_fees all ensure that the CollectedFeesPerBlock resource exists under cedra_framework by calling the is_fees_collection_enabled method, which returns a boolean value confirming if the resource exists or not.</td>
<td>Formally verified via <a href="#high-level-req-6.1">register_proposer_for_fee_collection</a>, <a href="#high-level-req-6.2">process_collected_fees</a>, and <a href="#high-level-req-6.3">upgrade_burn_percentage</a>.</td>
</tr>

</table>




<a id="module-level-spec"></a>

### Module-level Specification


<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
// This enforces <a id="high-level-req-1" href="#high-level-req">high-level requirement 1</a>:
<b>invariant</b> [suspendable] <a href="chain_status.md#0x1_chain_status_is_operating">chain_status::is_operating</a>() ==&gt; <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a>&gt;(@cedra_framework) || <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a>&gt;(@cedra_framework);
</code></pre>



<a id="@Specification_1_CollectedFeesPerBlock"></a>

### Resource `CollectedFeesPerBlock`


<pre><code>#[deprecated]
<b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> <b>has</b> key
</code></pre>



<dl>
<dt>
<code>amount: <a href="coin.md#0x1_coin_AggregatableCoin">coin::AggregatableCoin</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>burn_percentage: u8</code>
</dt>
<dd>

</dd>
</dl>



<pre><code>// This enforces <a id="high-level-req-4" href="#high-level-req">high-level requirement 4</a>:
<b>invariant</b> burn_percentage &lt;= 100;
</code></pre>



<a id="@Specification_1_burn_fee"></a>

### Function `burn_fee`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee">burn_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64)
</code></pre>


<code><a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a></code> should be exists.


<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a>&gt;(@cedra_framework);
<b>let</b> account_addr = <a href="account.md#0x1_account">account</a>;
<b>let</b> amount = fee;
<b>let</b> cedra_addr = <a href="../../cedra-stdlib/doc/type_info.md#0x1_type_info_type_of">type_info::type_of</a>&lt;CedraCoin&gt;().account_address;
<b>let</b> coin_store = <b>global</b>&lt;CoinStore&lt;CedraCoin&gt;&gt;(account_addr);
<b>let</b> <b>post</b> post_coin_store = <b>global</b>&lt;CoinStore&lt;CedraCoin&gt;&gt;(account_addr);
<b>aborts_if</b> amount != 0 && !(<b>exists</b>&lt;CoinInfo&lt;CedraCoin&gt;&gt;(cedra_addr)
    && <b>exists</b>&lt;CoinStore&lt;CedraCoin&gt;&gt;(account_addr));
<b>aborts_if</b> coin_store.<a href="coin.md#0x1_coin">coin</a>.value &lt; amount;
<b>let</b> maybe_supply = <b>global</b>&lt;CoinInfo&lt;CedraCoin&gt;&gt;(cedra_addr).supply;
<b>let</b> supply_aggr = <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_spec_borrow">option::spec_borrow</a>(maybe_supply);
<b>let</b> value = <a href="optional_aggregator.md#0x1_optional_aggregator_optional_aggregator_value">optional_aggregator::optional_aggregator_value</a>(supply_aggr);
<b>let</b> <b>post</b> post_maybe_supply = <b>global</b>&lt;CoinInfo&lt;CedraCoin&gt;&gt;(cedra_addr).supply;
<b>let</b> <b>post</b> post_supply = <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_spec_borrow">option::spec_borrow</a>(post_maybe_supply);
<b>let</b> <b>post</b> post_value = <a href="optional_aggregator.md#0x1_optional_aggregator_optional_aggregator_value">optional_aggregator::optional_aggregator_value</a>(post_supply);
<b>aborts_if</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_spec_is_some">option::spec_is_some</a>(maybe_supply) && value &lt; amount;
<b>ensures</b> post_coin_store.<a href="coin.md#0x1_coin">coin</a>.value == coin_store.<a href="coin.md#0x1_coin">coin</a>.value - amount;
<b>ensures</b> <b>if</b> (<a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_spec_is_some">option::spec_is_some</a>(maybe_supply)) {
    post_value == value - amount
} <b>else</b> {
    <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_spec_is_none">option::spec_is_none</a>(post_maybe_supply)
};
<b>ensures</b> <a href="coin.md#0x1_coin_supply">coin::supply</a>&lt;CedraCoin&gt; == <b>old</b>(<a href="coin.md#0x1_coin_supply">coin::supply</a>&lt;CedraCoin&gt;) - amount;
</code></pre>



<a id="@Specification_1_mint_and_refund"></a>

### Function `mint_and_refund`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_mint_and_refund">mint_and_refund</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, refund: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>let</b> cedra_addr = <a href="../../cedra-stdlib/doc/type_info.md#0x1_type_info_type_of">type_info::type_of</a>&lt;CedraCoin&gt;().account_address;
<b>aborts_if</b> (refund != 0) && !<b>exists</b>&lt;CoinInfo&lt;CedraCoin&gt;&gt;(cedra_addr);
<b>include</b> <a href="coin.md#0x1_coin_CoinAddAbortsIf">coin::CoinAddAbortsIf</a>&lt;CedraCoin&gt; { amount: refund };
<b>aborts_if</b> !<b>exists</b>&lt;CoinStore&lt;CedraCoin&gt;&gt;(<a href="account.md#0x1_account">account</a>);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a>&gt;(@cedra_framework);
<b>let</b> supply = <a href="coin.md#0x1_coin_supply">coin::supply</a>&lt;CedraCoin&gt;;
<b>let</b> <b>post</b> post_supply = <a href="coin.md#0x1_coin_supply">coin::supply</a>&lt;CedraCoin&gt;;
<b>aborts_if</b> [abstract] supply + refund &gt; MAX_U128;
<b>ensures</b> post_supply == supply + refund;
</code></pre>



<a id="@Specification_1_store_cedra_coin_burn_cap"></a>

### Function `store_cedra_coin_burn_cap`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_cedra_coin_burn_cap">store_cedra_coin_burn_cap</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, burn_cap: <a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;)
</code></pre>


Ensure caller is admin.
Aborts if <code><a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a></code> already exists.


<pre><code><b>pragma</b> verify = <b>false</b>;
<b>let</b> addr = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(cedra_framework);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_cedra_framework_address">system_addresses::is_cedra_framework_address</a>(addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a>&gt;(addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraFABurnCapabilities">CedraFABurnCapabilities</a>&gt;(addr) || <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinCapabilities">CedraCoinCapabilities</a>&gt;(addr);
</code></pre>



<a id="@Specification_1_store_cedra_coin_mint_cap"></a>

### Function `store_cedra_coin_mint_cap`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_cedra_coin_mint_cap">store_cedra_coin_mint_cap</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, mint_cap: <a href="coin.md#0x1_coin_MintCapability">coin::MintCapability</a>&lt;<a href="cedra_coin.md#0x1_cedra_coin_CedraCoin">cedra_coin::CedraCoin</a>&gt;)
</code></pre>


Ensure caller is admin.
Aborts if <code><a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a></code> already exists.


<pre><code><b>let</b> addr = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(cedra_framework);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_cedra_framework_address">system_addresses::is_cedra_framework_address</a>(addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CedraCoinMintCapability">CedraCoinMintCapability</a>&gt;(addr);
</code></pre>



<a id="@Specification_1_emit_fee_statement"></a>

### Function `emit_fee_statement`


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_emit_fee_statement">emit_fee_statement</a>(fee_statement: <a href="transaction_fee.md#0x1_transaction_fee_FeeStatement">transaction_fee::FeeStatement</a>)
</code></pre>


Aborts if module event feature is not enabled.


<a id="@Specification_1_initialize_fee_collection_and_distribution"></a>

### Function `initialize_fee_collection_and_distribution`


<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">initialize_fee_collection_and_distribution</a>(_cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _burn_percentage: u8)
</code></pre>




<a id="@Specification_1_initialize_storage_refund"></a>

### Function `initialize_storage_refund`


<pre><code>#[deprecated]
<b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_storage_refund">initialize_storage_refund</a>(_: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>


Historical. Aborts.


<pre><code><b>aborts_if</b> <b>true</b>;
</code></pre>


[move-book]: https://cedra.dev/move/book/SUMMARY
