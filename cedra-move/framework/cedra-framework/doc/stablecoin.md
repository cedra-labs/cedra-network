
<a id="0x1_stablecoin"></a>

# Module `0x1::stablecoin`



-  [Resource `Management`](#0x1_stablecoin_Management)
-  [Resource `Roles`](#0x1_stablecoin_Roles)
-  [Struct `Mint`](#0x1_stablecoin_Mint)
-  [Constants](#@Constants_0)
-  [Function `create`](#0x1_stablecoin_create)
-  [Function `mint`](#0x1_stablecoin_mint)
-  [Function `add_minter`](#0x1_stablecoin_add_minter)
-  [Function `add_minters`](#0x1_stablecoin_add_minters)
-  [Function `update_authorized_caller`](#0x1_stablecoin_update_authorized_caller)
-  [Function `update_authorized_callers`](#0x1_stablecoin_update_authorized_callers)
-  [Function `authorized_transfer`](#0x1_stablecoin_authorized_transfer)
-  [Function `asset_deployed`](#0x1_stablecoin_asset_deployed)
-  [Function `asset_address`](#0x1_stablecoin_asset_address)
-  [Function `metadata`](#0x1_stablecoin_metadata)
-  [Function `authorized_callers`](#0x1_stablecoin_authorized_callers)
-  [Function `balance`](#0x1_stablecoin_balance)


<pre><code><b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_stablecoin_Management"></a>

## Resource `Management`

Resource to control fungible assets refs.


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="stablecoin.md#0x1_stablecoin_Management">Management</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>transfer_ref: <a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a></code>
</dt>
<dd>

</dd>
<dt>
<code>mint_ref: <a href="fungible_asset.md#0x1_fungible_asset_MintRef">fungible_asset::MintRef</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stablecoin_Roles"></a>

## Resource `Roles`

Resource to control who can use fungible assets refs.


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>admin: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>authorized_callers: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>master_minter: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>minters: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stablecoin_Mint"></a>

## Struct `Mint`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="stablecoin.md#0x1_stablecoin_Mint">Mint</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>minter: <b>address</b></code>
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
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_stablecoin_EINSUFFICIENT_BALANCE"></a>



<pre><code><b>const</b> <a href="stablecoin.md#0x1_stablecoin_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>: u64 = 2;
</code></pre>



<a id="0x1_stablecoin_EALREADY_MINTER"></a>

Caller is already minter


<pre><code><b>const</b> <a href="stablecoin.md#0x1_stablecoin_EALREADY_MINTER">EALREADY_MINTER</a>: u64 = 3;
</code></pre>



<a id="0x1_stablecoin_EUNAUTHORIZED"></a>

Caller is not authorized to make this call


<pre><code><b>const</b> <a href="stablecoin.md#0x1_stablecoin_EUNAUTHORIZED">EUNAUTHORIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_stablecoin_create"></a>

## Function `create`

Create a new fungible asset with associated roles and management.


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_create">create</a>(deployer: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, name: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, decimals: u8, icon_url: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, project_url: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_create">create</a>(
    deployer: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    name: String,
    decimals: u8,
    icon_url: String,
    project_url: String
) {
    <b>let</b> deployer_addr = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(deployer);
    <b>let</b> constructor_ref = &<a href="object.md#0x1_object_create_named_object">object::create_named_object</a>(deployer, symbol);

    <a href="primary_fungible_store.md#0x1_primary_fungible_store_create_primary_store_enabled_fungible_asset">primary_fungible_store::create_primary_store_enabled_fungible_asset</a>(
        constructor_ref,
        <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_none">option::none</a>(),
        name,
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(symbol),
        decimals,
        icon_url,
        project_url
    );

    <b>move_to</b>(
        &<a href="object.md#0x1_object_generate_signer">object::generate_signer</a>(constructor_ref),
        <a href="stablecoin.md#0x1_stablecoin_Management">Management</a> {
            transfer_ref: <a href="fungible_asset.md#0x1_fungible_asset_generate_transfer_ref">fungible_asset::generate_transfer_ref</a>(constructor_ref),
            mint_ref: <a href="fungible_asset.md#0x1_fungible_asset_generate_mint_ref">fungible_asset::generate_mint_ref</a>(constructor_ref)
        }
    );

    <b>move_to</b>(
        &<a href="object.md#0x1_object_generate_signer">object::generate_signer</a>(constructor_ref),
        <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> {
            admin: @admin,
            authorized_callers: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_singleton">vector::singleton</a>(deployer_addr),
            master_minter: deployer_addr,
            minters: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_singleton">vector::singleton</a>(deployer_addr)
        }
    );

}
</code></pre>



</details>

<a id="0x1_stablecoin_mint"></a>

## Function `mint`

Mint new tokens to the specified account. Caller must be a minter.


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_mint">mint</a>(minter: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, creator_addr: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_mint">mint</a>(
    minter: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    creator_addr: <b>address</b>,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64
) <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>, <a href="stablecoin.md#0x1_stablecoin_Management">Management</a> {
    <b>if</b> (amount == 0) { <b>return</b> };

    <b>let</b> minter_addr = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(minter);
    <b>let</b> roles = <b>borrow_global</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_addr, symbol));

    <b>let</b> is_auth = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&roles.minters, &minter_addr);
    <b>assert</b>!(is_auth, <a href="stablecoin.md#0x1_stablecoin_EUNAUTHORIZED">EUNAUTHORIZED</a>);

    <b>let</b> management = <b>borrow_global</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Management">Management</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_addr, symbol));

    <a href="fungible_asset.md#0x1_fungible_asset_mint_to">fungible_asset::mint_to</a>(
        &management.mint_ref,
        std::primary_fungible_store::ensure_primary_store_exists(
            minter_addr, <a href="stablecoin.md#0x1_stablecoin_metadata">metadata</a>(creator_addr, symbol)
        ),
        amount
    );

    <a href="event.md#0x1_event_emit">event::emit</a>(<a href="stablecoin.md#0x1_stablecoin_Mint">Mint</a> { minter: minter_addr, <b>to</b>: creator_addr, amount });
}
</code></pre>



</details>

<a id="0x1_stablecoin_add_minter"></a>

## Function `add_minter`

Add a new minter. Must be called by the master minter.


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_add_minter">add_minter</a>(creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, minter: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_add_minter">add_minter</a>(
    creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, minter: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> {
    <b>let</b> creator_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(creator);
    <b>let</b> roles = <b>borrow_global_mut</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_address, symbol));

    <b>assert</b>!(creator_address == roles.master_minter, <a href="stablecoin.md#0x1_stablecoin_EUNAUTHORIZED">EUNAUTHORIZED</a>);
    <b>if</b> (<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&roles.minters, &minter)) { <b>return</b> };

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> roles.minters, minter);
}
</code></pre>



</details>

<a id="0x1_stablecoin_add_minters"></a>

## Function `add_minters`

Batch add multiple minters. Must be called by the master minter.


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_add_minters">add_minters</a>(creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, minters: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_add_minters">add_minters</a>(
    creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, minters: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> {
    <b>let</b> creator_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(creator);
    <b>let</b> roles = <b>borrow_global_mut</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_address, symbol));

    <b>assert</b>!(creator_address == roles.master_minter, <a href="stablecoin.md#0x1_stablecoin_EUNAUTHORIZED">EUNAUTHORIZED</a>);

    <b>let</b> len = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&minters);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> minter = *<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&minters, i);
        <b>if</b> (!<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&roles.minters, &minter)) {
            <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> roles.minters, minter);
        };
        i = i + 1;
    };
}
</code></pre>



</details>

<a id="0x1_stablecoin_update_authorized_caller"></a>

## Function `update_authorized_caller`

Add the account as an authorized caller.


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_update_authorized_caller">update_authorized_caller</a>(creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, authorized_caller: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_update_authorized_caller">update_authorized_caller</a>(
    creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, authorized_caller: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> {
    <b>let</b> creator_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(creator);
    <b>let</b> roles = <b>borrow_global_mut</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_address, symbol));

    <b>assert</b>!(creator_address == roles.master_minter, <a href="stablecoin.md#0x1_stablecoin_EUNAUTHORIZED">EUNAUTHORIZED</a>);
    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> roles.authorized_callers, authorized_caller);
}
</code></pre>



</details>

<a id="0x1_stablecoin_update_authorized_callers"></a>

## Function `update_authorized_callers`

Batch add multiple accounts as authorized callers.


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_update_authorized_callers">update_authorized_callers</a>(creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, authorized_callers: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_update_authorized_callers">update_authorized_callers</a>(
    creator: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, authorized_callers: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> {
    <b>let</b> creator_address = <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(creator);
    <b>let</b> roles = <b>borrow_global_mut</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_address, symbol));

    <b>assert</b>!(creator_address == roles.master_minter, <a href="stablecoin.md#0x1_stablecoin_EUNAUTHORIZED">EUNAUTHORIZED</a>);

    <b>let</b> len = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&authorized_callers);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> caller = *<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&authorized_callers, i);
        // Prevent duplicates
        <b>if</b> (!<a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&roles.authorized_callers, &caller)) {
            <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> roles.authorized_callers, caller);
        };
        i = i + 1;
    };
}
</code></pre>



</details>

<a id="0x1_stablecoin_authorized_transfer"></a>

## Function `authorized_transfer`

Transfer tokens with authorization check.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_authorized_transfer">authorized_transfer</a>(creator_addr: <b>address</b>, authorized_caller: <b>address</b>, from: <b>address</b>, <b>to</b>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_authorized_transfer">authorized_transfer</a>(
    creator_addr: <b>address</b>,
    authorized_caller: <b>address</b>,
    from: <b>address</b>,
    <b>to</b>: <b>address</b>,
    symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64
) <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>, <a href="stablecoin.md#0x1_stablecoin_Management">Management</a> {
    <b>if</b> (amount == 0) { <b>return</b> };

    <b>let</b> asset_addr = <a href="object.md#0x1_object_object_address">object::object_address</a>(&<a href="stablecoin.md#0x1_stablecoin_metadata">metadata</a>(creator_addr, symbol));
    <b>let</b> from_balance = <a href="stablecoin.md#0x1_stablecoin_balance">balance</a>(creator_addr, from, <b>copy</b> symbol);
    <b>assert</b>!(from_balance &gt;= amount, <a href="stablecoin.md#0x1_stablecoin_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>);

    <b>let</b> roles = <b>borrow_global</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(asset_addr);
    <b>let</b> management = <b>borrow_global</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Management">Management</a>&gt;(asset_addr);

    <b>let</b> is_auth = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_contains">vector::contains</a>(&roles.authorized_callers, &authorized_caller);
    <b>if</b> (!is_auth) {
        <b>return</b>;
    };

    <a href="primary_fungible_store.md#0x1_primary_fungible_store_transfer_with_ref">primary_fungible_store::transfer_with_ref</a>(
        &management.transfer_ref, from, <b>to</b>, amount
    );
}
</code></pre>



</details>

<a id="0x1_stablecoin_asset_deployed"></a>

## Function `asset_deployed`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_asset_deployed">asset_deployed</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_asset_deployed">asset_deployed</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool {
    <b>exists</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(owner, symbol))
}
</code></pre>



</details>

<a id="0x1_stablecoin_asset_address"></a>

## Function `asset_address`



<pre><code><b>fun</b> <a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(owner: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <b>address</b> {
    <a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&owner, symbol)
}
</code></pre>



</details>

<a id="0x1_stablecoin_metadata"></a>

## Function `metadata`



<pre><code><b>fun</b> <a href="stablecoin.md#0x1_stablecoin_metadata">metadata</a>(creator: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stablecoin.md#0x1_stablecoin_metadata">metadata</a>(creator: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): Object&lt;Metadata&gt; {
    <a href="object.md#0x1_object_address_to_object">object::address_to_object</a>&lt;Metadata&gt;(<a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator, symbol))
}
</code></pre>



</details>

<a id="0x1_stablecoin_authorized_callers"></a>

## Function `authorized_callers`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_authorized_callers">authorized_callers</a>(creator_address: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_authorized_callers">authorized_callers</a>(
    creator_address: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a> {
    <b>let</b> asset_addr = <a href="stablecoin.md#0x1_stablecoin_asset_address">asset_address</a>(creator_address, symbol);
    <b>borrow_global</b>&lt;<a href="stablecoin.md#0x1_stablecoin_Roles">Roles</a>&gt;(asset_addr).authorized_callers
}
</code></pre>



</details>

<a id="0x1_stablecoin_balance"></a>

## Function `balance`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_balance">balance</a>(admin: <b>address</b>, <a href="account.md#0x1_account">account</a>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stablecoin.md#0x1_stablecoin_balance">balance</a>(
    admin: <b>address</b>, <a href="account.md#0x1_account">account</a>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): u64 {
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_balance">primary_fungible_store::balance</a>(<a href="account.md#0x1_account">account</a>, <a href="stablecoin.md#0x1_stablecoin_metadata">metadata</a>(admin, symbol))
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
