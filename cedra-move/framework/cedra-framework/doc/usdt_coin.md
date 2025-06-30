
<a id="0x1_usdt_coin"></a>

# Module `0x1::usdt_coin`



-  [Resource `UsdtCoin`](#0x1_usdt_coin_UsdtCoin)
-  [Resource `MintCapStore`](#0x1_usdt_coin_MintCapStore)
-  [Struct `DelegatedMintCapability`](#0x1_usdt_coin_DelegatedMintCapability)
-  [Resource `Delegations`](#0x1_usdt_coin_Delegations)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_usdt_coin_initialize)
-  [Function `has_mint_capability`](#0x1_usdt_coin_has_mint_capability)
-  [Function `destroy_mint_cap`](#0x1_usdt_coin_destroy_mint_cap)
-  [Function `find_delegation`](#0x1_usdt_coin_find_delegation)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_usdt_coin_UsdtCoin"></a>

## Resource `UsdtCoin`



<pre><code><b>struct</b> <a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">UsdtCoin</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_usdt_coin_MintCapStore"></a>

## Resource `MintCapStore`



<pre><code><b>struct</b> <a href="usdt_coin.md#0x1_usdt_coin_MintCapStore">MintCapStore</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_cap: <a href="coin.md#0x1_coin_MintCapability">coin::MintCapability</a>&lt;<a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">usdt_coin::UsdtCoin</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_usdt_coin_DelegatedMintCapability"></a>

## Struct `DelegatedMintCapability`

Delegation token created by delegator and can be claimed by the delegatee as MintCapability.


<pre><code><b>struct</b> <a href="usdt_coin.md#0x1_usdt_coin_DelegatedMintCapability">DelegatedMintCapability</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><b>to</b>: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_usdt_coin_Delegations"></a>

## Resource `Delegations`

The container stores the current pending delegations.


<pre><code><b>struct</b> <a href="usdt_coin.md#0x1_usdt_coin_Delegations">Delegations</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>inner: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="usdt_coin.md#0x1_usdt_coin_DelegatedMintCapability">usdt_coin::DelegatedMintCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_usdt_coin_EALREADY_DELEGATED"></a>

Mint capability has already been delegated to this specified address


<pre><code><b>const</b> <a href="usdt_coin.md#0x1_usdt_coin_EALREADY_DELEGATED">EALREADY_DELEGATED</a>: u64 = 2;
</code></pre>



<a id="0x1_usdt_coin_EDELEGATION_NOT_FOUND"></a>

Cannot find delegation of mint capability to this account


<pre><code><b>const</b> <a href="usdt_coin.md#0x1_usdt_coin_EDELEGATION_NOT_FOUND">EDELEGATION_NOT_FOUND</a>: u64 = 3;
</code></pre>



<a id="0x1_usdt_coin_ENO_CAPABILITIES"></a>

Account does not have mint capability


<pre><code><b>const</b> <a href="usdt_coin.md#0x1_usdt_coin_ENO_CAPABILITIES">ENO_CAPABILITIES</a>: u64 = 1;
</code></pre>



<a id="0x1_usdt_coin_initialize"></a>

## Function `initialize`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>): (<a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">usdt_coin::UsdtCoin</a>&gt;, <a href="coin.md#0x1_coin_MintCapability">coin::MintCapability</a>&lt;<a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">usdt_coin::UsdtCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_initialize">initialize</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>): (BurnCapability&lt;<a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">UsdtCoin</a>&gt;, MintCapability&lt;<a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">UsdtCoin</a>&gt;) {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(cedra_framework);

    <b>let</b> (burn_cap, freeze_cap, mint_cap) = <a href="coin.md#0x1_coin_initialize_with_parallelizable_supply">coin::initialize_with_parallelizable_supply</a>&lt;<a href="usdt_coin.md#0x1_usdt_coin_UsdtCoin">UsdtCoin</a>&gt;(
        cedra_framework,
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"Usdt Coin"),
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"USDT"),
        8, // decimals
        <b>true</b>, // monitor_supply
    );

    // Cedra framework needs mint cap <b>to</b> mint coins <b>to</b> initial validators. This will be revoked once the validators
    // have been initialized.
    <b>move_to</b>(cedra_framework, <a href="usdt_coin.md#0x1_usdt_coin_MintCapStore">MintCapStore</a> { mint_cap });

    <a href="coin.md#0x1_coin_destroy_freeze_cap">coin::destroy_freeze_cap</a>(freeze_cap);
    (burn_cap, mint_cap)
}
</code></pre>



</details>

<a id="0x1_usdt_coin_has_mint_capability"></a>

## Function `has_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_has_mint_capability">has_mint_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_has_mint_capability">has_mint_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>): bool {
    <b>exists</b>&lt;<a href="usdt_coin.md#0x1_usdt_coin_MintCapStore">MintCapStore</a>&gt;(<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>))
}
</code></pre>



</details>

<a id="0x1_usdt_coin_destroy_mint_cap"></a>

## Function `destroy_mint_cap`

Only called during genesis to destroy the cedra framework account's mint capability once all initial validators
and accounts have been initialized during genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_destroy_mint_cap">destroy_mint_cap</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_destroy_mint_cap">destroy_mint_cap</a>(cedra_framework: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_MintCapStore">MintCapStore</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_cedra_framework">system_addresses::assert_cedra_framework</a>(cedra_framework);
    <b>let</b> <a href="usdt_coin.md#0x1_usdt_coin_MintCapStore">MintCapStore</a> { mint_cap } = <b>move_from</b>&lt;<a href="usdt_coin.md#0x1_usdt_coin_MintCapStore">MintCapStore</a>&gt;(@cedra_framework);
    <a href="coin.md#0x1_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_find_delegation"></a>

## Function `find_delegation`



<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_find_delegation">find_delegation</a>(addr: <b>address</b>): <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_find_delegation">find_delegation</a>(addr: <b>address</b>): Option&lt;u64&gt; <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_Delegations">Delegations</a> {
    <b>let</b> delegations = &<b>borrow_global</b>&lt;<a href="usdt_coin.md#0x1_usdt_coin_Delegations">Delegations</a>&gt;(@core_resources).inner;
    <b>let</b> i = 0;
    <b>let</b> len = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(delegations);
    <b>let</b> index = <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_none">option::none</a>();
    <b>while</b> (i &lt; len) {
        <b>let</b> element = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(delegations, i);
        <b>if</b> (element.<b>to</b> == addr) {
            index = <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(i);
            <b>break</b>
        };
        i = i + 1;
    };
    index
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
