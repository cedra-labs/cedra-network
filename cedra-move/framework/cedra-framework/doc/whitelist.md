
<a id="0x1_whitelist"></a>

# Module `0x1::whitelist`



-  [Struct `FungibleAssetStruct`](#0x1_whitelist_FungibleAssetStruct)
-  [Resource `FungibleAssetRegistry`](#0x1_whitelist_FungibleAssetRegistry)
-  [Struct `WhitelistAsset`](#0x1_whitelist_WhitelistAsset)
-  [Resource `WhitelistRegistry`](#0x1_whitelist_WhitelistRegistry)
-  [Struct `AssetAddedEvent`](#0x1_whitelist_AssetAddedEvent)
-  [Struct `AssetRemovedEvent`](#0x1_whitelist_AssetRemovedEvent)
-  [Constants](#@Constants_0)
-  [Function `init_registry`](#0x1_whitelist_init_registry)
-  [Function `migrate_registry`](#0x1_whitelist_migrate_registry)
-  [Function `destroy_registry`](#0x1_whitelist_destroy_registry)
-  [Function `add_asset`](#0x1_whitelist_add_asset)
-  [Function `remove_asset`](#0x1_whitelist_remove_asset)
-  [Function `add_cedra_coin`](#0x1_whitelist_add_cedra_coin)
-  [Function `asset_exists`](#0x1_whitelist_asset_exists)
-  [Function `has_registry`](#0x1_whitelist_has_registry)
-  [Function `is_cedra_coin`](#0x1_whitelist_is_cedra_coin)
-  [Function `is_migrated`](#0x1_whitelist_is_migrated)
-  [Function `get_canonical_asset_list`](#0x1_whitelist_get_canonical_asset_list)
-  [Function `get_asset_list`](#0x1_whitelist_get_asset_list)
-  [Function `ensure_canonical_registry`](#0x1_whitelist_ensure_canonical_registry)
-  [Function `migrate_registry_internal`](#0x1_whitelist_migrate_registry_internal)
-  [Function `legacy_assets_to_canonical`](#0x1_whitelist_legacy_assets_to_canonical)
-  [Function `canonical_assets_to_legacy`](#0x1_whitelist_canonical_assets_to_legacy)
-  [Function `to_legacy_struct`](#0x1_whitelist_to_legacy_struct)
-  [Function `oracle_module_name`](#0x1_whitelist_oracle_module_name)
-  [Function `asset_exists_canonical`](#0x1_whitelist_asset_exists_canonical)
-  [Function `asset_exists_legacy`](#0x1_whitelist_asset_exists_legacy)
-  [Function `assert_no_registry`](#0x1_whitelist_assert_no_registry)
-  [Function `cedra_coin_asset`](#0x1_whitelist_cedra_coin_asset)


<pre><code><b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stablecoin.md#0x1_stablecoin">0x1::stablecoin</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_whitelist_FungibleAssetStruct"></a>

## Struct `FungibleAssetStruct`



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

<a id="0x1_whitelist_FungibleAssetRegistry"></a>

## Resource `FungibleAssetRegistry`



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

<a id="0x1_whitelist_WhitelistAsset"></a>

## Struct `WhitelistAsset`



<pre><code><b>struct</b> <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> <b>has</b> <b>copy</b>, drop, store
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

<a id="0x1_whitelist_WhitelistRegistry"></a>

## Resource `WhitelistRegistry`



<pre><code><b>struct</b> <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>assets: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>&gt;</code>
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_whitelist_EUNAUTHORIZED"></a>

Caller is not authorized to make this call


<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_whitelist_CEDRA_COIN_MODULE"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_MODULE">CEDRA_COIN_MODULE</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [99, 101, 100, 114, 97, 95, 99, 111, 105, 110];
</code></pre>



<a id="0x1_whitelist_CEDRA_COIN_SYMBOL"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_SYMBOL">CEDRA_COIN_SYMBOL</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [67, 101, 100, 114, 97, 67, 111, 105, 110];
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



<a id="0x1_whitelist_ENO_LEGACY_REGISTRY"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_ENO_LEGACY_REGISTRY">ENO_LEGACY_REGISTRY</a>: u64 = 6;
</code></pre>



<a id="0x1_whitelist_ENO_REGISTRY"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_ENO_REGISTRY">ENO_REGISTRY</a>: u64 = 4;
</code></pre>



<a id="0x1_whitelist_STABLECOIN_MODULE"></a>



<pre><code><b>const</b> <a href="whitelist.md#0x1_whitelist_STABLECOIN_MODULE">STABLECOIN_MODULE</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [115, 116, 97, 98, 108, 101, 99, 111, 105, 110];
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
    <a href="whitelist.md#0x1_whitelist_assert_no_registry">assert_no_registry</a>(@admin);

    <b>let</b> assets = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt;();
    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> assets, <a href="whitelist.md#0x1_whitelist_cedra_coin_asset">cedra_coin_asset</a>());

    <b>move_to</b>(
        admin,
        <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
            assets
        }
    );

    emit(
        <a href="whitelist.md#0x1_whitelist_AssetAddedEvent">AssetAddedEvent</a> {
            addr: @0x1,
            module_name: <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_MODULE">CEDRA_COIN_MODULE</a>,
            symbol: <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_SYMBOL">CEDRA_COIN_SYMBOL</a>
        }
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_migrate_registry"></a>

## Function `migrate_registry`

Migrates the legacy registry into <code><a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a></code> and removes the old resource.


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_migrate_registry">migrate_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_migrate_registry">migrate_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>assert</b>!(@admin == admin_address, <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>);
    <b>assert</b>!(<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin), <a href="whitelist.md#0x1_whitelist_ENO_LEGACY_REGISTRY">ENO_LEGACY_REGISTRY</a>);
    <b>assert</b>!(!<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin), <a href="whitelist.md#0x1_whitelist_EALREADY_INITIALIZED">EALREADY_INITIALIZED</a>);
    <a href="whitelist.md#0x1_whitelist_migrate_registry_internal">migrate_registry_internal</a>(admin);
}
</code></pre>



</details>

<a id="0x1_whitelist_destroy_registry"></a>

## Function `destroy_registry`

Destroys both legacy and canonical registries.


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_destroy_registry">destroy_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_destroy_registry">destroy_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>assert</b>!(@admin == admin_address, <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>);
    <b>assert</b>!(<a href="whitelist.md#0x1_whitelist_has_registry">has_registry</a>(@admin), <a href="whitelist.md#0x1_whitelist_ENO_REGISTRY">ENO_REGISTRY</a>);

    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin)) {
        <b>let</b> <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> { assets: _ } = <b>move_from</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin);
    };
    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin)) {
        <b>let</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> { assets: _ } = <b>move_from</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin);
    };
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
) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);

    <a href="whitelist.md#0x1_whitelist_ensure_canonical_registry">ensure_canonical_registry</a>(admin);
    <b>assert</b>!(
        admin_address == @admin || admin_address == @0x1,
        <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>
    );

    <b>assert</b>!(
        <a href="stablecoin.md#0x1_stablecoin_asset_deployed">stablecoin::asset_deployed</a>(asset_addr, symbol),
        <a href="whitelist.md#0x1_whitelist_EASSET_NOT_FOUND">EASSET_NOT_FOUND</a>
    );

    <b>assert</b>!(
        !<a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(asset_addr, symbol),
        <a href="whitelist.md#0x1_whitelist_EASSET_EXISTS">EASSET_EXISTS</a>
    );

    <b>let</b> registry = <b>borrow_global_mut</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin);

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> registry.assets,
        <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> { addr: asset_addr, symbol }
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



<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_remove_asset">remove_asset</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, asset_addr: <b>address</b>, module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="whitelist.md#0x1_whitelist_remove_asset">remove_asset</a>(
    admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    asset_addr: <b>address</b>,
    module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>assert</b>!(@admin == admin_address, <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>);

    <a href="whitelist.md#0x1_whitelist_ensure_canonical_registry">ensure_canonical_registry</a>(admin);

    <b>let</b> registry = <b>borrow_global_mut</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin_address);

    <b>let</b> (exist, index) = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_index_of">vector::index_of</a>(
        &registry.assets,
        &<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> { addr: asset_addr, symbol }
    );
    <b>if</b> (exist) {
        <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_remove">vector::remove</a>(&<b>mut</b> registry.assets, index);

        emit(
            <a href="whitelist.md#0x1_whitelist_AssetRemovedEvent">AssetRemovedEvent</a> {
                addr: asset_addr,
                module_name,
                symbol
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
) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);

    <a href="whitelist.md#0x1_whitelist_ensure_canonical_registry">ensure_canonical_registry</a>(admin);
    <b>assert</b>!(
        admin_address == @admin || admin_address == @0x1,
        <a href="whitelist.md#0x1_whitelist_EUNAUTHORIZED">EUNAUTHORIZED</a>
    );

    <b>assert</b>!(
        !<a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(@0x1, <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_SYMBOL">CEDRA_COIN_SYMBOL</a>),
        <a href="whitelist.md#0x1_whitelist_EASSET_EXISTS">EASSET_EXISTS</a>
    );

    <b>let</b> registry = <b>borrow_global_mut</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin);

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> registry.assets,
        <a href="whitelist.md#0x1_whitelist_cedra_coin_asset">cedra_coin_asset</a>()
    );

    emit(
        <a href="whitelist.md#0x1_whitelist_AssetAddedEvent">AssetAddedEvent</a> {
            addr: @0x1,
            module_name: <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_MODULE">CEDRA_COIN_MODULE</a>,
            symbol: <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_SYMBOL">CEDRA_COIN_SYMBOL</a>
        }
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_asset_exists"></a>

## Function `asset_exists`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists">asset_exists</a>(
    asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin)) {
        <b>return</b> <a href="whitelist.md#0x1_whitelist_asset_exists_canonical">asset_exists_canonical</a>(asset_addr, symbol);
    };

    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin)) {
        <b>return</b> <a href="whitelist.md#0x1_whitelist_asset_exists_legacy">asset_exists_legacy</a>(asset_addr, symbol);
    };

    <b>false</b>
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
    <b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(addr) || <b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(addr)
}
</code></pre>



</details>

<a id="0x1_whitelist_is_cedra_coin"></a>

## Function `is_cedra_coin`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_is_cedra_coin">is_cedra_coin</a>(addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_is_cedra_coin">is_cedra_coin</a>(addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool {
    addr == @0x1 && symbol == <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_SYMBOL">CEDRA_COIN_SYMBOL</a>
}
</code></pre>



</details>

<a id="0x1_whitelist_is_migrated"></a>

## Function `is_migrated`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_is_migrated">is_migrated</a>(admin: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_is_migrated">is_migrated</a>(admin: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin)
}
</code></pre>



</details>

<a id="0x1_whitelist_get_canonical_asset_list"></a>

## Function `get_canonical_asset_list`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_get_canonical_asset_list">get_canonical_asset_list</a>(admin: <b>address</b>): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="whitelist.md#0x1_whitelist_get_canonical_asset_list">get_canonical_asset_list</a>(
    admin: <b>address</b>
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt; <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin)) {
        <b>return</b> <b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin).assets
    };

    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin)) {
        <b>return</b> <a href="whitelist.md#0x1_whitelist_legacy_assets_to_canonical">legacy_assets_to_canonical</a>(
            &<b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin).assets
        );
    };

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt;()
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
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt; <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>, <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin)) {
        <b>return</b> <a href="whitelist.md#0x1_whitelist_canonical_assets_to_legacy">canonical_assets_to_legacy</a>(
            &<b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin).assets
        );
    };

    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin)) {
        <b>return</b> <b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin).assets;
    };

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt;()
}
</code></pre>



</details>

<a id="0x1_whitelist_ensure_canonical_registry"></a>

## Function `ensure_canonical_registry`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_ensure_canonical_registry">ensure_canonical_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_ensure_canonical_registry">ensure_canonical_registry</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin_address)) {
        <b>return</b>
    };

    <b>if</b> (<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin_address)) {
        <a href="whitelist.md#0x1_whitelist_migrate_registry_internal">migrate_registry_internal</a>(admin);
        <b>return</b>
    };

    <b>abort</b> <a href="whitelist.md#0x1_whitelist_ENO_REGISTRY">ENO_REGISTRY</a>
}
</code></pre>



</details>

<a id="0x1_whitelist_migrate_registry_internal"></a>

## Function `migrate_registry_internal`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_migrate_registry_internal">migrate_registry_internal</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_migrate_registry_internal">migrate_registry_internal</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> admin_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(admin);
    <b>let</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> { assets: legacy_assets } =
        <b>move_from</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin_address);

    <b>let</b> canonical_assets = <a href="whitelist.md#0x1_whitelist_legacy_assets_to_canonical">legacy_assets_to_canonical</a>(&legacy_assets);

    <b>move_to</b>(
        admin,
        <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> { assets: canonical_assets }
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_legacy_assets_to_canonical"></a>

## Function `legacy_assets_to_canonical`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_legacy_assets_to_canonical">legacy_assets_to_canonical</a>(legacy_assets: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">whitelist::FungibleAssetStruct</a>&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_legacy_assets_to_canonical">legacy_assets_to_canonical</a>(
    legacy_assets: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt;
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt; {
    <b>let</b> canonical = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt;();
    <b>let</b> i = 0;
    <b>let</b> n = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(legacy_assets);
    <b>while</b> (i &lt; n) {
        <b>let</b> legacy = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(legacy_assets, i);
        <b>let</b> canonical_asset = <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> {
            addr: legacy.addr,
            symbol: legacy.symbol
        };
        <b>if</b> (!<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&canonical, &canonical_asset)) {
            <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> canonical, canonical_asset);
        };
        i = i + 1;
    };
    canonical
}
</code></pre>



</details>

<a id="0x1_whitelist_canonical_assets_to_legacy"></a>

## Function `canonical_assets_to_legacy`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_canonical_assets_to_legacy">canonical_assets_to_legacy</a>(canonical_assets: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">whitelist::FungibleAssetStruct</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_canonical_assets_to_legacy">canonical_assets_to_legacy</a>(
    canonical_assets: &<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>&gt;
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt; {
    <b>let</b> legacy = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a>&gt;();
    <b>let</b> i = 0;
    <b>let</b> n = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(canonical_assets);
    <b>while</b> (i &lt; n) {
        <b>let</b> asset = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(canonical_assets, i);
        <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> legacy, <a href="whitelist.md#0x1_whitelist_to_legacy_struct">to_legacy_struct</a>(asset));
        i = i + 1;
    };
    legacy
}
</code></pre>



</details>

<a id="0x1_whitelist_to_legacy_struct"></a>

## Function `to_legacy_struct`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_to_legacy_struct">to_legacy_struct</a>(asset: &<a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>): <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">whitelist::FungibleAssetStruct</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_to_legacy_struct">to_legacy_struct</a>(asset: &<a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a>): <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a> {
    <a href="whitelist.md#0x1_whitelist_FungibleAssetStruct">FungibleAssetStruct</a> {
        addr: asset.addr,
        module_name: <a href="whitelist.md#0x1_whitelist_oracle_module_name">oracle_module_name</a>(asset.addr, asset.symbol),
        symbol: asset.symbol
    }
}
</code></pre>



</details>

<a id="0x1_whitelist_oracle_module_name"></a>

## Function `oracle_module_name`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_oracle_module_name">oracle_module_name</a>(addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_oracle_module_name">oracle_module_name</a>(addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>if</b> (<a href="whitelist.md#0x1_whitelist_is_cedra_coin">is_cedra_coin</a>(addr, symbol)) {
        <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_MODULE">CEDRA_COIN_MODULE</a>
    } <b>else</b> {
        <a href="whitelist.md#0x1_whitelist_STABLECOIN_MODULE">STABLECOIN_MODULE</a>
    }
}
</code></pre>



</details>

<a id="0x1_whitelist_asset_exists_canonical"></a>

## Function `asset_exists_canonical`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists_canonical">asset_exists_canonical</a>(asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists_canonical">asset_exists_canonical</a>(
    asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool <b>acquires</b> <a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a> {
    <b>let</b> registry = <b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(@admin);
    <b>let</b> i = 0;
    <b>let</b> n = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&registry.assets);
    <b>while</b> (i &lt; n) {
        <b>let</b> asset = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&registry.assets, i);
        <b>if</b> (asset.addr == asset_addr && asset.symbol == symbol) {
            <b>return</b> <b>true</b>;
        };
        i = i + 1;
    };
    <b>false</b>
}
</code></pre>



</details>

<a id="0x1_whitelist_asset_exists_legacy"></a>

## Function `asset_exists_legacy`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists_legacy">asset_exists_legacy</a>(asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_asset_exists_legacy">asset_exists_legacy</a>(
    asset_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool <b>acquires</b> <a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a> {
    <b>let</b> registry = <b>borrow_global</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(@admin);
    <b>let</b> i = 0;
    <b>let</b> n = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&registry.assets);
    <b>while</b> (i &lt; n) {
        <b>let</b> asset = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&registry.assets, i);
        <b>if</b> (asset.addr == asset_addr && asset.symbol == symbol) {
            <b>return</b> <b>true</b>;
        };
        i = i + 1;
    };
    <b>false</b>
}
</code></pre>



</details>

<a id="0x1_whitelist_assert_no_registry"></a>

## Function `assert_no_registry`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_assert_no_registry">assert_no_registry</a>(admin_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_assert_no_registry">assert_no_registry</a>(admin_address: <b>address</b>) {
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_WhitelistRegistry">WhitelistRegistry</a>&gt;(admin_address)
            && !<b>exists</b>&lt;<a href="whitelist.md#0x1_whitelist_FungibleAssetRegistry">FungibleAssetRegistry</a>&gt;(admin_address),
        <a href="whitelist.md#0x1_whitelist_EALREADY_INITIALIZED">EALREADY_INITIALIZED</a>
    );
}
</code></pre>



</details>

<a id="0x1_whitelist_cedra_coin_asset"></a>

## Function `cedra_coin_asset`



<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_cedra_coin_asset">cedra_coin_asset</a>(): <a href="whitelist.md#0x1_whitelist_WhitelistAsset">whitelist::WhitelistAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="whitelist.md#0x1_whitelist_cedra_coin_asset">cedra_coin_asset</a>(): <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> {
    <a href="whitelist.md#0x1_whitelist_WhitelistAsset">WhitelistAsset</a> { addr: @0x1, symbol: <a href="whitelist.md#0x1_whitelist_CEDRA_COIN_SYMBOL">CEDRA_COIN_SYMBOL</a> }
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
