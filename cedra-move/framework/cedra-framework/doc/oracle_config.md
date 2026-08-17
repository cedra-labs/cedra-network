
<a id="0x1_oracle_config"></a>

# Module `0x1::oracle_config`

On-chain oracle module address used for FA fee pricing.
Set at genesis per network and updated only by framework governance.


-  [Resource `OracleConfig`](#0x1_oracle_config_OracleConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_oracle_config_initialize)
-  [Function `set`](#0x1_oracle_config_set)
-  [Function `set_for_next_epoch`](#0x1_oracle_config_set_for_next_epoch)
-  [Function `on_new_epoch`](#0x1_oracle_config_on_new_epoch)
-  [Function `oracle_address`](#0x1_oracle_config_oracle_address)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `set`](#@Specification_1_set)
    -  [Function `set_for_next_epoch`](#@Specification_1_set_for_next_epoch)
    -  [Function `on_new_epoch`](#@Specification_1_on_new_epoch)
    -  [Function `oracle_address`](#@Specification_1_oracle_address)


<pre><code><b>use</b> <a href="chain_status.md#0x1_chain_status">0x1::chain_status</a>;
<b>use</b> <a href="config_buffer.md#0x1_config_buffer">0x1::config_buffer</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_oracle_config_OracleConfig"></a>

## Resource `OracleConfig`



<pre><code><b>struct</b> <a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a> <b>has</b> drop, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_oracle_config_EINVALID_ORACLE_ADDRESS"></a>

Oracle address must be non-zero.


<pre><code><b>const</b> <a href="oracle_config.md#0x1_oracle_config_EINVALID_ORACLE_ADDRESS">EINVALID_ORACLE_ADDRESS</a>: u64 = 1;
</code></pre>



<a id="0x1_oracle_config_EORACLE_CONFIG_NOT_FOUND"></a>

Oracle config has not been initialized.


<pre><code><b>const</b> <a href="oracle_config.md#0x1_oracle_config_EORACLE_CONFIG_NOT_FOUND">EORACLE_CONFIG_NOT_FOUND</a>: u64 = 2;
</code></pre>



<a id="0x1_oracle_config_initialize"></a>

## Function `initialize`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(cedra_framework);
    <b>assert</b>!(oracle_addr != @0x0, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="oracle_config.md#0x1_oracle_config_EINVALID_ORACLE_ADDRESS">EINVALID_ORACLE_ADDRESS</a>));
    <b>move_to</b>(cedra_framework, <a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a> { addr: oracle_addr });
}
</code></pre>



</details>

<a id="0x1_oracle_config_set"></a>

## Function `set`

Genesis-only synchronous update. Prefer <code>set_for_next_epoch</code> after genesis.


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_set">set</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_set">set</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>) <b>acquires</b> <a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(<a href="account.md#0x1_account">account</a>);
    <a href="chain_status.md#0x1_chain_status_assert_genesis">chain_status::assert_genesis</a>();
    <b>assert</b>!(oracle_addr != @0x0, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="oracle_config.md#0x1_oracle_config_EINVALID_ORACLE_ADDRESS">EINVALID_ORACLE_ADDRESS</a>));
    <b>borrow_global_mut</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework).addr = oracle_addr;
}
</code></pre>



</details>

<a id="0x1_oracle_config_set_for_next_epoch"></a>

## Function `set_for_next_epoch`

Governance update applied at the next epoch boundary.

```
cedra_framework::oracle_config::set_for_next_epoch(&framework_signer, oracle_addr);
cedra_framework::cedra_governance::reconfigure(&framework_signer);
```


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_set_for_next_epoch">set_for_next_epoch</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_set_for_next_epoch">set_for_next_epoch</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(oracle_addr != @0x0, <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="oracle_config.md#0x1_oracle_config_EINVALID_ORACLE_ADDRESS">EINVALID_ORACLE_ADDRESS</a>));
    <a href="config_buffer.md#0x1_config_buffer_upsert">config_buffer::upsert</a>(<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a> { addr: oracle_addr });
}
</code></pre>



</details>

<a id="0x1_oracle_config_on_new_epoch"></a>

## Function `on_new_epoch`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_on_new_epoch">on_new_epoch</a>(framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_on_new_epoch">on_new_epoch</a>(framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(framework);
    <b>if</b> (<a href="config_buffer.md#0x1_config_buffer_does_exist">config_buffer::does_exist</a>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;()) {
        <b>let</b> new_config = <a href="config_buffer.md#0x1_config_buffer_extract_v2">config_buffer::extract_v2</a>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;();
        <b>if</b> (<b>exists</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework)) {
            *<b>borrow_global_mut</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework) = new_config;
        } <b>else</b> {
            <b>move_to</b>(framework, new_config);
        };
    }
}
</code></pre>



</details>

<a id="0x1_oracle_config_oracle_address"></a>

## Function `oracle_address`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_oracle_address">oracle_address</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_oracle_address">oracle_address</a>(): <b>address</b> <b>acquires</b> <a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework), <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="oracle_config.md#0x1_oracle_config_EORACLE_CONFIG_NOT_FOUND">EORACLE_CONFIG_NOT_FOUND</a>));
    <b>borrow_global</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework).addr
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict;
<b>invariant</b> [suspendable] <a href="chain_status.md#0x1_chain_status_is_operating">chain_status::is_operating</a>() ==&gt; <b>exists</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework);
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>)
</code></pre>




<pre><code><b>let</b> addr = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(cedra_framework);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_cedra_framework_address">system_addresses::is_cedra_framework_address</a>(addr);
<b>aborts_if</b> oracle_addr == @0x0;
<b>aborts_if</b> <b>exists</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework);
<b>ensures</b> <b>global</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(addr).addr == oracle_addr;
</code></pre>



<a id="@Specification_1_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_set">set</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
<b>let</b> addr = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_cedra_framework_address">system_addresses::is_cedra_framework_address</a>(addr);
<b>aborts_if</b> oracle_addr == @0x0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework);
<b>requires</b> <a href="chain_status.md#0x1_chain_status_is_genesis">chain_status::is_genesis</a>();
<b>ensures</b> <b>global</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework).addr == oracle_addr;
</code></pre>



<a id="@Specification_1_set_for_next_epoch"></a>

### Function `set_for_next_epoch`


<pre><code><b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_set_for_next_epoch">set_for_next_epoch</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, oracle_addr: <b>address</b>)
</code></pre>




<pre><code><b>let</b> addr = std::signer::address_of(<a href="account.md#0x1_account">account</a>);
<b>aborts_if</b> addr != @cedra_framework;
<b>aborts_if</b> oracle_addr == @0x0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="config_buffer.md#0x1_config_buffer_PendingConfigs">config_buffer::PendingConfigs</a>&gt;(@cedra_framework);
</code></pre>



<a id="@Specification_1_on_new_epoch"></a>

### Function `on_new_epoch`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_on_new_epoch">on_new_epoch</a>(framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>requires</b> @cedra_framework == std::signer::address_of(framework);
<b>include</b> <a href="config_buffer.md#0x1_config_buffer_OnNewEpochRequirement">config_buffer::OnNewEpochRequirement</a>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;;
<b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_1_oracle_address"></a>

### Function `oracle_address`


<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="oracle_config.md#0x1_oracle_config_oracle_address">oracle_address</a>(): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework);
<b>ensures</b> result == <b>global</b>&lt;<a href="oracle_config.md#0x1_oracle_config_OracleConfig">OracleConfig</a>&gt;(@cedra_framework).addr;
</code></pre>


[move-book]: https://cedra.dev/move/book/SUMMARY
