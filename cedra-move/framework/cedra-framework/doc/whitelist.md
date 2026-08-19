
<a id="0x1_whitelist"></a>

# Module `0x1::whitelist`



-  [Resource `FungibleAssetRegistry`](#0x1_whitelist_FungibleAssetRegistry)
-  [Struct `FungibleAssetStruct`](#0x1_whitelist_FungibleAssetStruct)
-  [Struct `AssetAddedEvent`](#0x1_whitelist_AssetAddedEvent)
-  [Struct `AssetRemovedEvent`](#0x1_whitelist_AssetRemovedEvent)
-  [Struct `WhitelistAsset`](#0x1_whitelist_WhitelistAsset)
-  [Constants](#@Constants_0)
-  [Function `init_registry`](#0x1_whitelist_init_registry)
-  [Function `add_asset`](#0x1_whitelist_add_asset)
-  [Function `remove_asset`](#0x1_whitelist_remove_asset)
-  [Function `add_cedra_coin`](#0x1_whitelist_add_cedra_coin)
-  [Function `asset_exists`](#0x1_whitelist_asset_exists)
-  [Function `find_asset_index`](#0x1_whitelist_find_asset_index)
-  [Function `has_registry`](#0x1_whitelist_has_registry)
-  [Function `assert_registry_absent`](#0x1_whitelist_assert_registry_absent)
-  [Function `get_asset_list`](#0x1_whitelist_get_asset_list)
-  [Function `get_asset_list_v2`](#0x1_whitelist_get_asset_list_v2)


<pre><code><b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stablecoin.md#0x1_stablecoin">0x1::stablecoin</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_whitelist_FungibleAssetRegistry"></a>

## Resource `FungibleAssetRegistry`

Stores all assets that allowed in transaction commission


<pre><code><b>struct</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>assets: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">whitelist::FungibleAssetStruct</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_whitelist_FungibleAssetStruct"></a>

## Struct `FungibleAssetStruct`

Stores Asset values.
<code>module_name</code> is retained so existing on-chain registry data can still be loaded;
identity is <code>(addr, symbol)</code> only.


<pre><code><b>struct</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_whitelist_AssetAddedEvent"></a>

## Struct `AssetAddedEvent`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="whitelist.md#0x1_whitelist_AssetAddedEvent">AssetAddedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_whitelist_AssetRemovedEvent"></a>

## Struct `AssetRemovedEvent`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="whitelist.md#0x1_whitelist_AssetRemovedEvent">AssetRemovedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_whitelist_WhitelistAsset"></a>

## Struct `WhitelistAsset`

View projection of a whitelist entry. Omits the legacy <code>module_name</code> storage field.


<pre><code><b>struct</b> <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_whitelist_EUNAUTHORIZED"></a>

Caller is not authorized to make this call


<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_whitelist_EALREADY_INITIALIZED"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_EALREADY_INITIALIZED">EALREADY_INITIALIZED</a>: u64 = 3;
</code></pre>



<a id="0x1_whitelist_EASSET_EXISTS"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_EASSET_EXISTS">EASSET_EXISTS</a>: u64 = 5;
</code></pre>



<a id="0x1_whitelist_EASSET_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_EASSET_NOT_FOUND">EASSET_NOT_FOUND</a>: u64 = 2;
</code></pre>



<a id="0x1_whitelist_ENO_REGISTRY"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_ENO_REGISTRY">ENO_REGISTRY</a>: u64 = 4;
</code></pre>



<a id="0x1_whitelist_init_registry"></a>

## Function `init_registry`



<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_init_registry">init_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_init_registry">init_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>assert</b>!(@admin == admin_address, <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>);

    <a href="whitelist.md#0x1_whitelist_assert_registry_absent">assert_registry_absent</a>(@admin);

    <b>let</b> assets = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt;();

    // Add default asset: 0x1 CedraCoin. module_name kept for on-chain layout compatibility.
    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> assets,
        <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a> {
            addr: @0x1,
            module_name: b"<a href="cedra_coin.md#0x1_cedra_coin">cedra_coin</a>",
            symbol: b"CedraCoin"
        }
    );

    <b>move_to</b>(
        admin,
        <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
            assets
        }
    );

    emit(
        <a href="whitelist.md#0x1_whitelist_AssetAddedEvent">AssetAddedEvent</a> {
            addr: @0x1,
            module_name: b"<a href="cedra_coin.md#0x1_cedra_coin">cedra_coin</a>",
            symbol: b"CedraCoin"
        }
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_add_asset"></a>

## Function `add_asset`



<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_add_asset">add_asset</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, asset_addr: <b>address</b>, module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_add_asset">add_asset</a>(
    admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    asset_addr: <b>address</b>,
    module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);

    <b>assert</b>!(<a href="whitelist.md#0x1_whitelist_has_registry">has_registry</a>(@admin), <a href="whitelist.md#0x1_whitelist_ENO_REGISTRY">ENO_REGISTRY</a>);
    <b>assert</b>!(
        admin_address == @admin || admin_address == @0x1,
        <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>
    );

    <b>assert</b>!(
        <a href="stablecoin.md#0x1_stablecoin_asset_deployed">stablecoin::asset_deployed</a>(asset_addr, symbol),
        <a href="whitelist.md#0x1_whitelist_EASSET_NOT_FOUND">EASSET_NOT_FOUND</a>
    );

    <b>assert</b>!(
        !<a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(asset_addr, <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>(), symbol),
        <a href="whitelist.md#0x1_whitelist_EASSET_EXISTS">EASSET_EXISTS</a>
    );

    <b>let</b> registry = <b>borrow_global_mut</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin);

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> registry.assets,
        <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a> { addr: asset_addr, module_name, symbol }
    );

     emit(
        <a href="whitelist.md#0x1_whitelist_AssetAddedEvent">AssetAddedEvent</a> {
            addr: asset_addr,
            module_name,
            symbol
        }
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_remove_asset"></a>

## Function `remove_asset`



<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_remove_asset">remove_asset</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, asset_addr: <b>address</b>, _module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_remove_asset">remove_asset</a>(
    admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    asset_addr: <b>address</b>,
    _module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>assert</b>!(@admin == admin_address, <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>);

    <b>let</b> registry = <b>borrow_global_mut</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin_address);

    <b>let</b> (exist, index) = <a href="whitelist.md#0x1_whitelist_find_asset_index">find_asset_index</a>(&registry.assets, asset_addr, symbol);
    <b>if</b> (exist) {
        <b>let</b> removed = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_remove">vector::remove</a>(&<b>mut</b> registry.assets, index);

       emit(
            <a href="whitelist.md#0x1_whitelist_AssetRemovedEvent">AssetRemovedEvent</a> {
                addr: removed.addr,
                module_name: removed.module_name,
                symbol: removed.symbol
            }
        );
    } <b>else</b> {
        <b>abort</b> <a href="whitelist.md#0x1_whitelist_EASSET_NOT_FOUND">EASSET_NOT_FOUND</a>
    }
}
</code></pre>



</details>

<a id="0x1_whitelist_add_cedra_coin"></a>

## Function `add_cedra_coin`



<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_add_cedra_coin">add_cedra_coin</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_add_cedra_coin">add_cedra_coin</a>(
    admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);

    <b>assert</b>!(<a href="whitelist.md#0x1_whitelist_has_registry">has_registry</a>(@admin), <a href="whitelist.md#0x1_whitelist_ENO_REGISTRY">ENO_REGISTRY</a>);
    <b>assert</b>!(
        admin_address == @admin || admin_address == @0x1,
        <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>
    );

    <b>assert</b>!(
        !<a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(@0x1, <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>(), b"CedraCoin"),
        <a href="whitelist.md#0x1_whitelist_EASSET_EXISTS">EASSET_EXISTS</a>
    );

    <b>let</b> registry = <b>borrow_global_mut</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin);

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> registry.assets,
        <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a> { addr: @0x1, module_name: b"<a href="cedra_coin.md#0x1_cedra_coin">cedra_coin</a>", symbol: b"CedraCoin"}
    );

    emit(
        <a href="whitelist.md#0x1_whitelist_AssetAddedEvent">AssetAddedEvent</a> {
            addr: @0x1,
            module_name: b"<a href="cedra_coin.md#0x1_cedra_coin">cedra_coin</a>",
            symbol: b"CedraCoin"
        }
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_asset_exists"></a>

## Function `asset_exists`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(asset_addr: <b>address</b>, _module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(
    asset_addr: <b>address</b>, _module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> registry = <b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin);
    <b>let</b> (exist, _) = <a href="whitelist.md#0x1_whitelist_find_asset_index">find_asset_index</a>(&registry.assets, asset_addr, symbol);
    exist
}
</code></pre>



</details>

<a id="0x1_whitelist_find_asset_index"></a>

## Function `find_asset_index`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_find_asset_index">find_asset_index</a>(assets: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">whitelist::FungibleAssetStruct</a>&gt;, asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_find_asset_index">find_asset_index</a>(
    assets: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt;, asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): (bool, u64) {
    <b>let</b> i = 0;
    <b>let</b> n = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(assets);
    <b>while</b> (i &lt; n) {
        <b>let</b> asset = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(assets, i);
        <b>if</b> (asset.addr == asset_addr && asset.symbol == symbol) {
            <b>return</b> (<b>true</b>, i);
        };
        i = i + 1;
    };
    (<b>false</b>, 0)
}
</code></pre>



</details>

<a id="0x1_whitelist_has_registry"></a>

## Function `has_registry`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="whitelist.md#0x1_whitelist_has_registry">has_registry</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="whitelist.md#0x1_whitelist_has_registry">has_registry</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(addr)
}
</code></pre>



</details>

<a id="0x1_whitelist_assert_registry_absent"></a>

## Function `assert_registry_absent`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_assert_registry_absent">assert_registry_absent</a>(admin_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_assert_registry_absent">assert_registry_absent</a>(admin_address: <b>address</b>) {
    <b>assert</b>!(!<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin_address), <a href="whitelist.md#0x1_whitelist_EALREADY_INITIALIZED">EALREADY_INITIALIZED</a>);
}
</code></pre>



</details>

<a id="0x1_whitelist_get_asset_list"></a>

## Function `get_asset_list`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_get_asset_list">get_asset_list</a>(admin: <b>address</b>): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">whitelist::FungibleAssetStruct</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_get_asset_list">get_asset_list</a>(
    admin: <b>address</b>
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt; <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin).assets
}
</code></pre>



</details>

<a id="0x1_whitelist_get_asset_list_v2"></a>

## Function `get_asset_list_v2`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_get_asset_list_v2">get_asset_list_v2</a>(admin: <b>address</b>): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_get_asset_list_v2">get_asset_list_v2</a>(
    admin: <b>address</b>
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt; <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> stored = &<b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin).assets;
    <b>let</b> out = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt;();
    <b>let</b> i = 0;
    <b>let</b> n = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(stored);
    <b>while</b> (i &lt; n) {
        <b>let</b> asset = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(stored, i);
        <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
            &<b>mut</b> out,
            <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> { addr: asset.addr, symbol: asset.symbol }
        );
        i = i + 1;
    };
    out
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
