
<a id="0x1_bridge"></a>

# Module `0x1::bridge`



-  [Resource `Admin`](#0x1_bridge_Admin)
-  [Resource `Config`](#0x1_bridge_Config)
-  [Struct `FACaps`](#0x1_bridge_FACaps)
-  [Resource `FARegistry`](#0x1_bridge_FARegistry)
-  [Struct `WithdrawalApproval`](#0x1_bridge_WithdrawalApproval)
-  [Resource `Requests`](#0x1_bridge_Requests)
-  [Struct `DepositObserved`](#0x1_bridge_DepositObserved)
-  [Struct `MintExecuted`](#0x1_bridge_MintExecuted)
-  [Struct `Withdrawal`](#0x1_bridge_Withdrawal)
-  [Resource `BridgeEvents`](#0x1_bridge_BridgeEvents)
-  [Constants](#@Constants_0)
-  [Function `assert_not_paused`](#0x1_bridge_assert_not_paused)
-  [Function `assert_framework`](#0x1_bridge_assert_framework)
-  [Function `assert_multisig`](#0x1_bridge_assert_multisig)
-  [Function `assert_20_bytes`](#0x1_bridge_assert_20_bytes)
-  [Function `get_registry`](#0x1_bridge_get_registry)
-  [Function `get_metadata_or_abort`](#0x1_bridge_get_metadata_or_abort)
-  [Function `get_caps_or_abort`](#0x1_bridge_get_caps_or_abort)
-  [Function `initialize`](#0x1_bridge_initialize)
-  [Function `set_multisig_framework_only`](#0x1_bridge_set_multisig_framework_only)
-  [Function `rotate_multisig`](#0x1_bridge_rotate_multisig)
-  [Function `pause`](#0x1_bridge_pause)
-  [Function `unpause`](#0x1_bridge_unpause)
-  [Function `add_asset`](#0x1_bridge_add_asset)
-  [Function `remove_asset`](#0x1_bridge_remove_asset)
-  [Function `execute_deposit`](#0x1_bridge_execute_deposit)
-  [Function `approve_withdrawal`](#0x1_bridge_approve_withdrawal)
-  [Function `withdraw_to_l1`](#0x1_bridge_withdraw_to_l1)
-  [Function `admin_multisig`](#0x1_bridge_admin_multisig)
-  [Function `nonce_used`](#0x1_bridge_nonce_used)
-  [Function `balance_of`](#0x1_bridge_balance_of)
-  [Function `ensure_store`](#0x1_bridge_ensure_store)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features">0x1::features</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../cedra-stdlib/doc/table.md#0x1_table">0x1::table</a>;
</code></pre>



<a id="0x1_bridge_Admin"></a>

## Resource `Admin`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_Admin">Admin</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>multisig: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_Config"></a>

## Resource `Config`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_Config">Config</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>paused: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_FACaps"></a>

## Struct `FACaps`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_FACaps">FACaps</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint: <a href="fungible_asset.md#0x1_fungible_asset_MintRef">fungible_asset::MintRef</a></code>
</dt>
<dd>

</dd>
<dt>
<code>burn: <a href="fungible_asset.md#0x1_fungible_asset_BurnRef">fungible_asset::BurnRef</a></code>
</dt>
<dd>

</dd>
<dt>
<code>transfer: <a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_FARegistry"></a>

## Resource `FARegistry`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>l1_to_metadata: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>caps_by_meta: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;<b>address</b>, <a href="bridge.md#0x1_bridge_FACaps">bridge::FACaps</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_WithdrawalApproval"></a>

## Struct `WithdrawalApproval`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_WithdrawalApproval">WithdrawalApproval</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>user: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>metadata_addr: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>eth_recipient: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_Requests"></a>

## Resource `Requests`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_Requests">Requests</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>used_nonce: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;u64, bool&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>approvals: <a href="../../cedra-stdlib/doc/table.md#0x1_table_Table">table::Table</a>&lt;u64, <a href="bridge.md#0x1_bridge_WithdrawalApproval">bridge::WithdrawalApproval</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_DepositObserved"></a>

## Struct `DepositObserved`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="bridge.md#0x1_bridge_DepositObserved">DepositObserved</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>eth_tx_hash: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code><b>to</b>: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>nonce: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_MintExecuted"></a>

## Struct `MintExecuted`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="bridge.md#0x1_bridge_MintExecuted">MintExecuted</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code><b>to</b>: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>nonce: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_Withdrawal"></a>

## Struct `Withdrawal`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="bridge.md#0x1_bridge_Withdrawal">Withdrawal</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>from: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>eth_recipient: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>nonce: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_bridge_BridgeEvents"></a>

## Resource `BridgeEvents`



<pre><code><b>struct</b> <a href="bridge.md#0x1_bridge_BridgeEvents">BridgeEvents</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>deposit_observed: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="bridge.md#0x1_bridge_DepositObserved">bridge::DepositObserved</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>mint_executed: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="bridge.md#0x1_bridge_MintExecuted">bridge::MintExecuted</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>withdrawal: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="bridge.md#0x1_bridge_Withdrawal">bridge::Withdrawal</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_bridge_BAD_L1"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_BAD_L1">BAD_L1</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57];
</code></pre>



<a id="0x1_bridge_DAI_L1"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_DAI_L1">DAI_L1</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50];
</code></pre>



<a id="0x1_bridge_ETH_L1"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_ETH_L1">ETH_L1</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48];
</code></pre>



<a id="0x1_bridge_E_ALREADY_ACTIVE"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_ALREADY_ACTIVE">E_ALREADY_ACTIVE</a>: u64 = 5;
</code></pre>



<a id="0x1_bridge_E_ALREADY_INITIALIZED"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_ALREADY_INITIALIZED">E_ALREADY_INITIALIZED</a>: u64 = 13;
</code></pre>



<a id="0x1_bridge_E_APPROVAL_MISMATCH"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_APPROVAL_MISMATCH">E_APPROVAL_MISMATCH</a>: u64 = 15;
</code></pre>



<a id="0x1_bridge_E_ASSET_EXISTS"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_ASSET_EXISTS">E_ASSET_EXISTS</a>: u64 = 16;
</code></pre>



<a id="0x1_bridge_E_ASSET_MISMATCH"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_ASSET_MISMATCH">E_ASSET_MISMATCH</a>: u64 = 18;
</code></pre>



<a id="0x1_bridge_E_ASSET_UNKNOWN"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_ASSET_UNKNOWN">E_ASSET_UNKNOWN</a>: u64 = 17;
</code></pre>



<a id="0x1_bridge_E_BAD_INPUT"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_BAD_INPUT">E_BAD_INPUT</a>: u64 = 2;
</code></pre>



<a id="0x1_bridge_E_NONCE_USED"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_NONCE_USED">E_NONCE_USED</a>: u64 = 9;
</code></pre>



<a id="0x1_bridge_E_NOT_ADMIN"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_NOT_ADMIN">E_NOT_ADMIN</a>: u64 = 1;
</code></pre>



<a id="0x1_bridge_E_NO_APPROVAL"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_NO_APPROVAL">E_NO_APPROVAL</a>: u64 = 14;
</code></pre>



<a id="0x1_bridge_E_PAUSED"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_PAUSED">E_PAUSED</a>: u64 = 11;
</code></pre>



<a id="0x1_bridge_E_ZERO_AMOUNT"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_E_ZERO_AMOUNT">E_ZERO_AMOUNT</a>: u64 = 12;
</code></pre>



<a id="0x1_bridge_USDC_L1"></a>



<pre><code><b>const</b> <a href="bridge.md#0x1_bridge_USDC_L1">USDC_L1</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49];
</code></pre>



<a id="0x1_bridge_assert_not_paused"></a>

## Function `assert_not_paused`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_assert_not_paused">assert_not_paused</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_assert_not_paused">assert_not_paused</a>() <b>acquires</b> <a href="bridge.md#0x1_bridge_Config">Config</a> {
    <b>assert</b>!(!<b>borrow_global</b>&lt;<a href="bridge.md#0x1_bridge_Config">Config</a>&gt;(@cedra_framework).paused, <a href="bridge.md#0x1_bridge_E_PAUSED">E_PAUSED</a>);
}
</code></pre>



</details>

<a id="0x1_bridge_assert_framework"></a>

## Function `assert_framework`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_assert_framework">assert_framework</a>(s: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_assert_framework">assert_framework</a>(s: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) { <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(s); }
</code></pre>



</details>

<a id="0x1_bridge_assert_multisig"></a>

## Function `assert_multisig`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(s: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(s: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <b>let</b> admin = <b>borrow_global</b>&lt;<a href="bridge.md#0x1_bridge_Admin">Admin</a>&gt;(@cedra_framework);
    <b>assert</b>!(<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(s) == admin.multisig, <a href="bridge.md#0x1_bridge_E_NOT_ADMIN">E_NOT_ADMIN</a>);
}
</code></pre>



</details>

<a id="0x1_bridge_assert_20_bytes"></a>

## Function `assert_20_bytes`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(addr: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(addr: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>assert</b>!(<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(addr) == 20, <a href="bridge.md#0x1_bridge_E_BAD_INPUT">E_BAD_INPUT</a>);
}
</code></pre>



</details>

<a id="0x1_bridge_get_registry"></a>

## Function `get_registry`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_get_registry">get_registry</a>(): &<b>mut</b> <a href="bridge.md#0x1_bridge_FARegistry">bridge::FARegistry</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_get_registry">get_registry</a>(): &<b>mut</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
    <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>&gt;(@cedra_framework)
}
</code></pre>



</details>

<a id="0x1_bridge_get_metadata_or_abort"></a>

## Function `get_metadata_or_abort`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_get_metadata_or_abort">get_metadata_or_abort</a>(l1: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_get_metadata_or_abort">get_metadata_or_abort</a>(l1: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): Object&lt;Metadata&gt; <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
    <b>let</b> reg = <b>borrow_global</b>&lt;<a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>&gt;(@cedra_framework);
    <b>assert</b>!(<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reg.l1_to_metadata, *l1), <a href="bridge.md#0x1_bridge_E_ASSET_UNKNOWN">E_ASSET_UNKNOWN</a>);
    *<a href="../../cedra-stdlib/doc/table.md#0x1_table_borrow">table::borrow</a>(&reg.l1_to_metadata, *l1)
}
</code></pre>



</details>

<a id="0x1_bridge_get_caps_or_abort"></a>

## Function `get_caps_or_abort`



<pre><code><b>fun</b> <a href="bridge.md#0x1_bridge_get_caps_or_abort">get_caps_or_abort</a>(meta_addr: <b>address</b>): &<b>mut</b> <a href="bridge.md#0x1_bridge_FACaps">bridge::FACaps</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="bridge.md#0x1_bridge_get_caps_or_abort">get_caps_or_abort</a>(meta_addr: <b>address</b>): &<b>mut</b> <a href="bridge.md#0x1_bridge_FACaps">FACaps</a> <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
    <b>let</b> reg = <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>&gt;(@cedra_framework);
    <b>assert</b>!(<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reg.caps_by_meta, meta_addr), <a href="bridge.md#0x1_bridge_E_ASSET_UNKNOWN">E_ASSET_UNKNOWN</a>);
    <a href="../../cedra-stdlib/doc/table.md#0x1_table_borrow_mut">table::borrow_mut</a>(&<b>mut</b> reg.caps_by_meta, meta_addr)
}
</code></pre>



</details>

<a id="0x1_bridge_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="bridge.md#0x1_bridge_assert_framework">assert_framework</a>(cedra_framework);
    <b>assert</b>!(!<b>exists</b>&lt;<a href="bridge.md#0x1_bridge_Config">Config</a>&gt;(@cedra_framework), <a href="bridge.md#0x1_bridge_E_ALREADY_INITIALIZED">E_ALREADY_INITIALIZED</a>);

    <b>move_to</b>(cedra_framework, <a href="bridge.md#0x1_bridge_Config">Config</a> { paused: <b>false</b> });

    <b>move_to</b>(cedra_framework, <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
        l1_to_metadata: <a href="../../cedra-stdlib/doc/table.md#0x1_table_new">table::new</a>&lt;<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, Object&lt;Metadata&gt;&gt;(),
        caps_by_meta:   <a href="../../cedra-stdlib/doc/table.md#0x1_table_new">table::new</a>&lt;<b>address</b>, <a href="bridge.md#0x1_bridge_FACaps">FACaps</a>&gt;(),
    });

    <b>move_to</b>(cedra_framework, <a href="bridge.md#0x1_bridge_Requests">Requests</a> {
        used_nonce: <a href="../../cedra-stdlib/doc/table.md#0x1_table_new">table::new</a>&lt;u64, bool&gt;(),
        approvals:  <a href="../../cedra-stdlib/doc/table.md#0x1_table_new">table::new</a>&lt;u64, <a href="bridge.md#0x1_bridge_WithdrawalApproval">WithdrawalApproval</a>&gt;(),
    });

    <b>move_to</b>(cedra_framework, <a href="bridge.md#0x1_bridge_BridgeEvents">BridgeEvents</a> {
        deposit_observed: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="bridge.md#0x1_bridge_DepositObserved">DepositObserved</a>&gt;(cedra_framework),
        mint_executed:    <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="bridge.md#0x1_bridge_MintExecuted">MintExecuted</a>&gt;(cedra_framework),
        withdrawal:       <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="bridge.md#0x1_bridge_Withdrawal">Withdrawal</a>&gt;(cedra_framework),
    });

    <b>move_to</b>(cedra_framework, <a href="bridge.md#0x1_bridge_Admin">Admin</a> { multisig: <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(cedra_framework) });
}
</code></pre>



</details>

<a id="0x1_bridge_set_multisig_framework_only"></a>

## Function `set_multisig_framework_only`



<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_set_multisig_framework_only">set_multisig_framework_only</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_set_multisig_framework_only">set_multisig_framework_only</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, addr: <b>address</b>) <b>acquires</b> <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <a href="bridge.md#0x1_bridge_assert_framework">assert_framework</a>(cedra_framework);
    <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Admin">Admin</a>&gt;(@cedra_framework).multisig = addr;
}
</code></pre>



</details>

<a id="0x1_bridge_rotate_multisig"></a>

## Function `rotate_multisig`



<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_rotate_multisig">rotate_multisig</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, new_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_rotate_multisig">rotate_multisig</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, new_addr: <b>address</b>) <b>acquires</b> <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(multisig);
    <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Admin">Admin</a>&gt;(@cedra_framework).multisig = new_addr;
}
</code></pre>



</details>

<a id="0x1_bridge_pause"></a>

## Function `pause`



<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_pause">pause</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_pause">pause</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="bridge.md#0x1_bridge_Config">Config</a>, <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(multisig);
    <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Config">Config</a>&gt;(@cedra_framework).paused = <b>true</b>;
}
</code></pre>



</details>

<a id="0x1_bridge_unpause"></a>

## Function `unpause`



<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_unpause">unpause</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_unpause">unpause</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="bridge.md#0x1_bridge_Config">Config</a>, <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(multisig);
    <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Config">Config</a>&gt;(@cedra_framework).paused = <b>false</b>;
}
</code></pre>



</details>

<a id="0x1_bridge_add_asset"></a>

## Function `add_asset`

Create a new FA for an L1 token and store metadata + refs.
All string-ish arguments are UTF-8 bytes (converted internally).


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_add_asset">add_asset</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, decimals: u8, icon_uri: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, project_uri: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_add_asset">add_asset</a>(
    cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    decimals: u8,
    icon_uri: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    project_uri: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
    <a href="bridge.md#0x1_bridge_assert_framework">assert_framework</a>(cedra_framework);
    <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(&l1_token);

    <b>let</b> reg = <a href="bridge.md#0x1_bridge_get_registry">get_registry</a>();
    <b>assert</b>!(!<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reg.l1_to_metadata, l1_token), <a href="bridge.md#0x1_bridge_E_ASSET_EXISTS">E_ASSET_EXISTS</a>);

    // Create a non-deletable <a href="object.md#0x1_object">object</a>; we can <b>use</b> the symbol (or l1_token) <b>as</b> <a href="object.md#0x1_object">object</a> name.
    <b>let</b> ctor = &<a href="object.md#0x1_object_create_named_object">object::create_named_object</a>(cedra_framework, symbol);

    // Create FA metadata and enable primary store auto-creation
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_create_primary_store_enabled_fungible_asset">primary_fungible_store::create_primary_store_enabled_fungible_asset</a>(
        ctor,
        <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;u128&gt;(),
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(name),
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(symbol),
        decimals,
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(icon_uri),
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(project_uri),
    );

    // Reconstruct the Metadata <a href="object.md#0x1_object">object</a> for this FA
    <b>let</b> meta: Object&lt;Metadata&gt; = <a href="object.md#0x1_object_object_from_constructor_ref">object::object_from_constructor_ref</a>&lt;Metadata&gt;(ctor);

    // Generate refs at creation time
    <b>let</b> mint_ref     = fa::generate_mint_ref(ctor);
    <b>let</b> transfer_ref = fa::generate_transfer_ref(ctor);
    <b>let</b> burn_ref     = fa::generate_burn_ref(ctor);

    // Record in registry
    <a href="../../cedra-stdlib/doc/table.md#0x1_table_add">table::add</a>(&<b>mut</b> reg.l1_to_metadata, l1_token, meta);
    <a href="../../cedra-stdlib/doc/table.md#0x1_table_add">table::add</a>(
        &<b>mut</b> reg.caps_by_meta,
        <a href="object.md#0x1_object_object_address">object::object_address</a>(&meta),
        <a href="bridge.md#0x1_bridge_FACaps">FACaps</a> { mint: mint_ref, burn: burn_ref, transfer: transfer_ref }
    );
}
</code></pre>



</details>

<a id="0x1_bridge_remove_asset"></a>

## Function `remove_asset`

Remove the L1->FA mapping (does not destroy metadata or refs).


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_remove_asset">remove_asset</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_remove_asset">remove_asset</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
    <a href="bridge.md#0x1_bridge_assert_framework">assert_framework</a>(cedra_framework);
    <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(&l1_token);

    <b>let</b> reg = <a href="bridge.md#0x1_bridge_get_registry">get_registry</a>();
    <b>assert</b>!(<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reg.l1_to_metadata, l1_token), <a href="bridge.md#0x1_bridge_E_ASSET_UNKNOWN">E_ASSET_UNKNOWN</a>);
    <b>let</b> _ = <a href="../../cedra-stdlib/doc/table.md#0x1_table_remove">table::remove</a>(&<b>mut</b> reg.l1_to_metadata, l1_token);
    // We deliberately keep caps entry so remaining holders can still operate/burn <b>if</b> you want that.
    // If you want <b>to</b> <b>freeze</b> <b>post</b>-delist, you can <b>use</b> transfer_ref <b>to</b> <b>freeze</b> accounts externally.
}
</code></pre>



</details>

<a id="0x1_bridge_execute_deposit"></a>

## Function `execute_deposit`



<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_execute_deposit">execute_deposit</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <b>to</b>: <b>address</b>, amount: u64, nonce: u64, eth_tx_hash: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_execute_deposit">execute_deposit</a>(
    multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <b>to</b>: <b>address</b>,
    amount: u64,
    nonce: u64,
    eth_tx_hash: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>, <a href="bridge.md#0x1_bridge_Requests">Requests</a>, <a href="bridge.md#0x1_bridge_BridgeEvents">BridgeEvents</a>, <a href="bridge.md#0x1_bridge_Config">Config</a>, <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <a href="bridge.md#0x1_bridge_assert_not_paused">assert_not_paused</a>();
    <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(multisig);
    <b>assert</b>!(amount &gt; 0, <a href="bridge.md#0x1_bridge_E_ZERO_AMOUNT">E_ZERO_AMOUNT</a>);
    <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(&l1_token);

    <b>let</b> reqs = <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Requests">Requests</a>&gt;(@cedra_framework);
    <b>assert</b>!(!<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reqs.used_nonce, nonce), <a href="bridge.md#0x1_bridge_E_NONCE_USED">E_NONCE_USED</a>);

    <b>let</b> meta = <a href="bridge.md#0x1_bridge_get_metadata_or_abort">get_metadata_or_abort</a>(&l1_token);
    <b>let</b> caps = <a href="bridge.md#0x1_bridge_get_caps_or_abort">get_caps_or_abort</a>(<a href="object.md#0x1_object_object_address">object::object_address</a>(&meta));

    <b>let</b> minted: FungibleAsset = fa::mint(&caps.mint, amount);
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_deposit">primary_fungible_store::deposit</a>(<b>to</b>, minted);

    <a href="../../cedra-stdlib/doc/table.md#0x1_table_add">table::add</a>(&<b>mut</b> reqs.used_nonce, nonce, <b>true</b>);

    <b>let</b> evs = <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_BridgeEvents">BridgeEvents</a>&gt;(@cedra_framework);
    <b>if</b> (<a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features_module_event_migration_enabled">features::module_event_migration_enabled</a>()) {
        ev::emit(<a href="bridge.md#0x1_bridge_DepositObserved">DepositObserved</a> { l1_token, eth_tx_hash, <b>to</b>, amount, nonce });
        ev::emit(<a href="bridge.md#0x1_bridge_MintExecuted">MintExecuted</a>    { l1_token, <b>to</b>, amount, nonce });
    } <b>else</b> {
        ev::emit_event(&<b>mut</b> evs.deposit_observed, <a href="bridge.md#0x1_bridge_DepositObserved">DepositObserved</a> { l1_token: l1_token, eth_tx_hash, <b>to</b>, amount, nonce });
        ev::emit_event(&<b>mut</b> evs.mint_executed,   <a href="bridge.md#0x1_bridge_MintExecuted">MintExecuted</a>    { l1_token: l1_token, <b>to</b>, amount, nonce });
    };
}
</code></pre>



</details>

<a id="0x1_bridge_approve_withdrawal"></a>

## Function `approve_withdrawal`



<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_approve_withdrawal">approve_withdrawal</a>(multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, user: <b>address</b>, l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, eth_recipient: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64, nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_approve_withdrawal">approve_withdrawal</a>(
    multisig: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    user: <b>address</b>,
    l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    eth_recipient: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64,
    nonce: u64
) <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>, <a href="bridge.md#0x1_bridge_Admin">Admin</a>, <a href="bridge.md#0x1_bridge_Requests">Requests</a>, <a href="bridge.md#0x1_bridge_Config">Config</a> {
    <a href="bridge.md#0x1_bridge_assert_not_paused">assert_not_paused</a>();
    <a href="bridge.md#0x1_bridge_assert_multisig">assert_multisig</a>(multisig);
    <b>assert</b>!(amount &gt; 0, <a href="bridge.md#0x1_bridge_E_ZERO_AMOUNT">E_ZERO_AMOUNT</a>);
    <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(&l1_token);
    <b>assert</b>!(<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&eth_recipient) == 20, <a href="bridge.md#0x1_bridge_E_BAD_INPUT">E_BAD_INPUT</a>);

    <b>let</b> reqs = <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Requests">Requests</a>&gt;(@cedra_framework);
    <b>assert</b>!(!<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reqs.used_nonce, nonce), <a href="bridge.md#0x1_bridge_E_NONCE_USED">E_NONCE_USED</a>);
    <b>assert</b>!(!<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reqs.approvals, nonce), <a href="bridge.md#0x1_bridge_E_ALREADY_ACTIVE">E_ALREADY_ACTIVE</a>);

    <b>let</b> meta = <a href="bridge.md#0x1_bridge_get_metadata_or_abort">get_metadata_or_abort</a>(&l1_token);
    <b>let</b> metadata_addr = <a href="object.md#0x1_object_object_address">object::object_address</a>(&meta);

    <a href="../../cedra-stdlib/doc/table.md#0x1_table_add">table::add</a>(&<b>mut</b> reqs.approvals, nonce, <a href="bridge.md#0x1_bridge_WithdrawalApproval">WithdrawalApproval</a> {
        user,
        l1_token,
        metadata_addr,
        eth_recipient,
        amount,
    });
}
</code></pre>



</details>

<a id="0x1_bridge_withdraw_to_l1"></a>

## Function `withdraw_to_l1`



<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_withdraw_to_l1">withdraw_to_l1</a>(user: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, eth_recipient: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64, nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_withdraw_to_l1">withdraw_to_l1</a>(
    user: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    l1_token: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    eth_recipient: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64,
    nonce: u64
) <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>, <a href="bridge.md#0x1_bridge_BridgeEvents">BridgeEvents</a>, <a href="bridge.md#0x1_bridge_Requests">Requests</a>, <a href="bridge.md#0x1_bridge_Config">Config</a> {
    <a href="bridge.md#0x1_bridge_assert_not_paused">assert_not_paused</a>();
    <b>assert</b>!(amount &gt; 0, <a href="bridge.md#0x1_bridge_E_ZERO_AMOUNT">E_ZERO_AMOUNT</a>);
    <b>assert</b>!(<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&eth_recipient) == 20, <a href="bridge.md#0x1_bridge_E_BAD_INPUT">E_BAD_INPUT</a>);
    <a href="bridge.md#0x1_bridge_assert_20_bytes">assert_20_bytes</a>(&l1_token);

    <b>let</b> reqs = <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_Requests">Requests</a>&gt;(@cedra_framework);
    <b>assert</b>!(<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reqs.approvals, nonce), <a href="bridge.md#0x1_bridge_E_NO_APPROVAL">E_NO_APPROVAL</a>);
    <b>let</b> appr = <a href="../../cedra-stdlib/doc/table.md#0x1_table_borrow">table::borrow</a>(&reqs.approvals, nonce);

    // Full match
    <b>assert</b>!(appr.user == <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(user), <a href="bridge.md#0x1_bridge_E_APPROVAL_MISMATCH">E_APPROVAL_MISMATCH</a>);
    <b>assert</b>!(appr.amount == amount, <a href="bridge.md#0x1_bridge_E_APPROVAL_MISMATCH">E_APPROVAL_MISMATCH</a>);
    <b>assert</b>!(appr.eth_recipient == eth_recipient, <a href="bridge.md#0x1_bridge_E_APPROVAL_MISMATCH">E_APPROVAL_MISMATCH</a>);
    <b>assert</b>!(appr.l1_token == l1_token, <a href="bridge.md#0x1_bridge_E_APPROVAL_MISMATCH">E_APPROVAL_MISMATCH</a>);

    // Re-fetch metadata from registry and ensure it's the same <a href="object.md#0x1_object">object</a> <b>as</b> approval
    <b>let</b> meta = <a href="bridge.md#0x1_bridge_get_metadata_or_abort">get_metadata_or_abort</a>(&l1_token);
    <b>let</b> meta_addr_now = <a href="object.md#0x1_object_object_address">object::object_address</a>(&meta);
    <b>assert</b>!(meta_addr_now == appr.metadata_addr, <a href="bridge.md#0x1_bridge_E_ASSET_MISMATCH">E_ASSET_MISMATCH</a>);

    // Withdraw from user's primary store, then burn <b>with</b> stored BurnRef
    <b>let</b> fa_withdrawn: FungibleAsset = <a href="primary_fungible_store.md#0x1_primary_fungible_store_withdraw">primary_fungible_store::withdraw</a>&lt;Metadata&gt;(user, meta, amount);
    <b>let</b> caps = <a href="bridge.md#0x1_bridge_get_caps_or_abort">get_caps_or_abort</a>(meta_addr_now);
    fa::burn(&caps.burn, fa_withdrawn);

    // Mark nonce used + clear approval
    <b>assert</b>!(!<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reqs.used_nonce, nonce), <a href="bridge.md#0x1_bridge_E_NONCE_USED">E_NONCE_USED</a>);
    <a href="../../cedra-stdlib/doc/table.md#0x1_table_add">table::add</a>(&<b>mut</b> reqs.used_nonce, nonce, <b>true</b>);
    <b>let</b> _ = <a href="../../cedra-stdlib/doc/table.md#0x1_table_remove">table::remove</a>(&<b>mut</b> reqs.approvals, nonce);

    <b>let</b> evs = <b>borrow_global_mut</b>&lt;<a href="bridge.md#0x1_bridge_BridgeEvents">BridgeEvents</a>&gt;(@cedra_framework);
    <b>if</b> (<a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features_module_event_migration_enabled">features::module_event_migration_enabled</a>()) {
        ev::emit(<a href="bridge.md#0x1_bridge_Withdrawal">Withdrawal</a> { l1_token, from: <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(user), eth_recipient, amount, nonce });
    } <b>else</b> {
        ev::emit_event(&<b>mut</b> evs.withdrawal, <a href="bridge.md#0x1_bridge_Withdrawal">Withdrawal</a> { l1_token, from: <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(user), eth_recipient, amount, nonce });
    };
}
</code></pre>



</details>

<a id="0x1_bridge_admin_multisig"></a>

## Function `admin_multisig`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_admin_multisig">admin_multisig</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_admin_multisig">admin_multisig</a>(): <b>address</b> <b>acquires</b> <a href="bridge.md#0x1_bridge_Admin">Admin</a> {
    <b>borrow_global</b>&lt;<a href="bridge.md#0x1_bridge_Admin">Admin</a>&gt;(@cedra_framework).multisig
}
</code></pre>



</details>

<a id="0x1_bridge_nonce_used"></a>

## Function `nonce_used`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_nonce_used">nonce_used</a>(n: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_nonce_used">nonce_used</a>(n: u64): bool <b>acquires</b> <a href="bridge.md#0x1_bridge_Requests">Requests</a> {
    <a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&<b>borrow_global</b>&lt;<a href="bridge.md#0x1_bridge_Requests">Requests</a>&gt;(@cedra_framework).used_nonce, n)
}
</code></pre>



</details>

<a id="0x1_bridge_balance_of"></a>

## Function `balance_of`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_balance_of">balance_of</a>(l1: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, owner: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="bridge.md#0x1_bridge_balance_of">balance_of</a>(l1: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, owner: <b>address</b>): u64 <b>acquires</b> <a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a> {
    <b>let</b> reg = <b>borrow_global</b>&lt;<a href="bridge.md#0x1_bridge_FARegistry">FARegistry</a>&gt;(@cedra_framework);
    // Use the <a href="../../cedra-stdlib/doc/table.md#0x1_table">table</a> via `reg`, never by value
    <b>assert</b>!(<a href="../../cedra-stdlib/doc/table.md#0x1_table_contains">table::contains</a>(&reg.l1_to_metadata, l1), <a href="bridge.md#0x1_bridge_E_ASSET_UNKNOWN">E_ASSET_UNKNOWN</a>);
    <b>let</b> meta_obj: Object&lt;Metadata&gt; = *<a href="../../cedra-stdlib/doc/table.md#0x1_table_borrow">table::borrow</a>(&reg.l1_to_metadata, l1);
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_balance">primary_fungible_store::balance</a>&lt;Metadata&gt;(owner, meta_obj)
}
</code></pre>



</details>

<a id="0x1_bridge_ensure_store"></a>

## Function `ensure_store`

No-op with FA: primary store is auto-created on demand. Left for CLI parity.


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_ensure_store">ensure_store</a>(_user: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="bridge.md#0x1_bridge_ensure_store">ensure_store</a>(_user: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
