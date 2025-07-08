
<a id="0x1_usdt_coin"></a>

# Module `0x1::usdt_coin`



-  [Resource `ManagedFungibleAsset`](#0x1_usdt_coin_ManagedFungibleAsset)
-  [Resource `State`](#0x1_usdt_coin_State)
-  [Constants](#@Constants_0)
-  [Function `init_module`](#0x1_usdt_coin_init_module)
-  [Function `get_metadata`](#0x1_usdt_coin_get_metadata)
-  [Function `deposit`](#0x1_usdt_coin_deposit)
-  [Function `withdraw`](#0x1_usdt_coin_withdraw)
-  [Function `mint`](#0x1_usdt_coin_mint)
-  [Function `transfer`](#0x1_usdt_coin_transfer)
-  [Function `burn`](#0x1_usdt_coin_burn)
-  [Function `freeze_account`](#0x1_usdt_coin_freeze_account)
-  [Function `unfreeze_account`](#0x1_usdt_coin_unfreeze_account)
-  [Function `set_pause`](#0x1_usdt_coin_set_pause)
-  [Function `assert_not_paused`](#0x1_usdt_coin_assert_not_paused)
-  [Function `authorized_borrow_refs`](#0x1_usdt_coin_authorized_borrow_refs)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="dispatchable_fungible_asset.md#0x1_dispatchable_fungible_asset">0x1::dispatchable_fungible_asset</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="function_info.md#0x1_function_info">0x1::function_info</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
</code></pre>



<a id="0x1_usdt_coin_ManagedFungibleAsset"></a>

## Resource `ManagedFungibleAsset`

Hold refs to control the minting, transfer and burning of fungible assets.


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_ref: <a href="fungible_asset.md#0x1_fungible_asset_MintRef">fungible_asset::MintRef</a></code>
</dt>
<dd>

</dd>
<dt>
<code>transfer_ref: <a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a></code>
</dt>
<dd>

</dd>
<dt>
<code>burn_ref: <a href="fungible_asset.md#0x1_fungible_asset_BurnRef">fungible_asset::BurnRef</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_usdt_coin_State"></a>

## Resource `State`

Global state to pause the FA coin.
OPTIONAL


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> <b>has</b> key
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_usdt_coin_ENOT_OWNER"></a>

Only fungible asset metadata owner can make changes.


<pre><code><b>const</b> <a href="usdt_coin.md#0x1_usdt_coin_ENOT_OWNER">ENOT_OWNER</a>: u64 = 1;
</code></pre>



<a id="0x1_usdt_coin_ASSET_SYMBOL"></a>



<pre><code><b>const</b> <a href="usdt_coin.md#0x1_usdt_coin_ASSET_SYMBOL">ASSET_SYMBOL</a>: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [85, 83, 68, 84];
</code></pre>



<a id="0x1_usdt_coin_EPAUSED"></a>

The FA coin is paused.


<pre><code><b>const</b> <a href="usdt_coin.md#0x1_usdt_coin_EPAUSED">EPAUSED</a>: u64 = 2;
</code></pre>



<a id="0x1_usdt_coin_init_module"></a>

## Function `init_module`

Initialize metadata object and store the refs.


<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_init_module">init_module</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_init_module">init_module</a>() {
    <b>let</b> (cedra_framework_account, cedra_framework_signer_cap) = <a href="account.md#0x1_account_create_framework_reserved_account">account::create_framework_reserved_account</a>(@cedra_framework);
    <b>let</b> constructor_ref = &<a href="object.md#0x1_object_create_named_object">object::create_named_object</a>(&cedra_framework_account, <a href="usdt_coin.md#0x1_usdt_coin_ASSET_SYMBOL">ASSET_SYMBOL</a>);
    <a href="primary_fungible_store.md#0x1_primary_fungible_store_create_primary_store_enabled_fungible_asset">primary_fungible_store::create_primary_store_enabled_fungible_asset</a>(
        constructor_ref,
        <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_none">option::none</a>(),
        utf8(b"Usdt Coin"), /* name */
        utf8(<a href="usdt_coin.md#0x1_usdt_coin_ASSET_SYMBOL">ASSET_SYMBOL</a>), /* symbol */
        8, /* decimals */
        utf8(b"http://example.com/favicon.ico"), /* icon */
        utf8(b"http://example.com"), /* project */
    );

    // Create mint/burn/transfer refs <b>to</b> allow creator <b>to</b> manage the fungible asset.
    <b>let</b> mint_ref = <a href="fungible_asset.md#0x1_fungible_asset_generate_mint_ref">fungible_asset::generate_mint_ref</a>(constructor_ref);
    <b>let</b> burn_ref = <a href="fungible_asset.md#0x1_fungible_asset_generate_burn_ref">fungible_asset::generate_burn_ref</a>(constructor_ref);
    <b>let</b> transfer_ref = <a href="fungible_asset.md#0x1_fungible_asset_generate_transfer_ref">fungible_asset::generate_transfer_ref</a>(constructor_ref);
    <b>let</b> metadata_object_signer = <a href="object.md#0x1_object_generate_signer">object::generate_signer</a>(constructor_ref);
    <b>move_to</b>(
        &metadata_object_signer,
        <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> { mint_ref, transfer_ref, burn_ref }
    ); // &lt;:!:initialize

    // Create a <b>global</b> state <b>to</b> pause the FA <a href="coin.md#0x1_coin">coin</a> and <b>move</b> <b>to</b> Metadata <a href="object.md#0x1_object">object</a>.
    <b>move_to</b>(
        &metadata_object_signer,
        <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> { paused: <b>false</b>, }
    );

    // Override the deposit and withdraw functions which mean overriding transfer.
    // This <b>ensures</b> all transfer will call withdraw and deposit functions in this <b>module</b>
    // and perform the necessary checks.
    // This is OPTIONAL. It is an advanced feature and we don't NEED a <b>global</b> state <b>to</b> pause the FA <a href="coin.md#0x1_coin">coin</a>.
    <b>let</b> deposit = <a href="function_info.md#0x1_function_info_new_function_info">function_info::new_function_info</a>(
        &cedra_framework_account,
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="usdt_coin.md#0x1_usdt_coin">usdt_coin</a>"),
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"deposit"),
    );
    <b>let</b> withdraw = <a href="function_info.md#0x1_function_info_new_function_info">function_info::new_function_info</a>(
        &cedra_framework_account,
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="usdt_coin.md#0x1_usdt_coin">usdt_coin</a>"),
        <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"withdraw"),
    );
    <a href="dispatchable_fungible_asset.md#0x1_dispatchable_fungible_asset_register_dispatch_functions">dispatchable_fungible_asset::register_dispatch_functions</a>(
        constructor_ref,
        <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(withdraw),
        <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(deposit),
        <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option_none">option::none</a>(),
    );
}
</code></pre>



</details>

<a id="0x1_usdt_coin_get_metadata"></a>

## Function `get_metadata`

Return the address of the managed fungible asset that's created when this module is deployed.


<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>(): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>(): Object&lt;Metadata&gt; {
    <b>let</b> asset_address = <a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&@cedra_framework, <a href="usdt_coin.md#0x1_usdt_coin_ASSET_SYMBOL">ASSET_SYMBOL</a>);
    <a href="object.md#0x1_object_address_to_object">object::address_to_object</a>&lt;Metadata&gt;(asset_address)
}
</code></pre>



</details>

<a id="0x1_usdt_coin_deposit"></a>

## Function `deposit`

Deposit function override to ensure that the account is not denylisted and the FA coin is not paused.
OPTIONAL


<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_deposit">deposit</a>&lt;T: key&gt;(store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;T&gt;, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>, transfer_ref: &<a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_deposit">deposit</a>&lt;T: key&gt;(
    store: Object&lt;T&gt;,
    fa: FungibleAsset,
    transfer_ref: &TransferRef,
) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> {
    <a href="usdt_coin.md#0x1_usdt_coin_assert_not_paused">assert_not_paused</a>();
    <a href="fungible_asset.md#0x1_fungible_asset_deposit_with_ref">fungible_asset::deposit_with_ref</a>(transfer_ref, store, fa);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_withdraw"></a>

## Function `withdraw`

Withdraw function override to ensure that the account is not denylisted and the FA coin is not paused.
OPTIONAL


<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_withdraw">withdraw</a>&lt;T: key&gt;(store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;T&gt;, amount: u64, transfer_ref: &<a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a>): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_withdraw">withdraw</a>&lt;T: key&gt;(
    store: Object&lt;T&gt;,
    amount: u64,
    transfer_ref: &TransferRef,
): FungibleAsset <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> {
    <a href="usdt_coin.md#0x1_usdt_coin_assert_not_paused">assert_not_paused</a>();
    <a href="fungible_asset.md#0x1_fungible_asset_withdraw_with_ref">fungible_asset::withdraw_with_ref</a>(transfer_ref, store, amount)
}
</code></pre>



</details>

<a id="0x1_usdt_coin_mint"></a>

## Function `mint`

Mint as the owner of metadata object.


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_mint">mint</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <b>to</b>: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_mint">mint</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <b>to</b>: <b>address</b>, amount: u64) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> {
    <b>let</b> asset = <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>();
    <b>let</b> managed_fungible_asset = <a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(admin, asset);
    <b>let</b> to_wallet = <a href="primary_fungible_store.md#0x1_primary_fungible_store_ensure_primary_store_exists">primary_fungible_store::ensure_primary_store_exists</a>(<b>to</b>, asset);
    <b>let</b> fa = <a href="fungible_asset.md#0x1_fungible_asset_mint">fungible_asset::mint</a>(&managed_fungible_asset.mint_ref, amount);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit_with_ref">fungible_asset::deposit_with_ref</a>(&managed_fungible_asset.transfer_ref, to_wallet, fa);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_transfer"></a>

## Function `transfer`

Transfer as the owner of metadata object ignoring <code>frozen</code> field.


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_transfer">transfer</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_transfer">transfer</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u64) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a>, <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> {
    <b>let</b> asset = <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>();
    <b>let</b> transfer_ref = &<a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(admin, asset).transfer_ref;
    <b>let</b> from_wallet = <a href="primary_fungible_store.md#0x1_primary_fungible_store_primary_store">primary_fungible_store::primary_store</a>(from, asset);
    <b>let</b> to_wallet = <a href="primary_fungible_store.md#0x1_primary_fungible_store_ensure_primary_store_exists">primary_fungible_store::ensure_primary_store_exists</a>(<b>to</b>, asset);
    <b>let</b> fa = <a href="usdt_coin.md#0x1_usdt_coin_withdraw">withdraw</a>(from_wallet, amount, transfer_ref);
    <a href="usdt_coin.md#0x1_usdt_coin_deposit">deposit</a>(to_wallet, fa, transfer_ref);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_burn"></a>

## Function `burn`

Burn fungible assets as the owner of metadata object.


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_burn">burn</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, from: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_burn">burn</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, from: <b>address</b>, amount: u64) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> {
    <b>let</b> asset = <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>();
    <b>let</b> burn_ref = &<a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(admin, asset).burn_ref;
    <b>let</b> from_wallet = <a href="primary_fungible_store.md#0x1_primary_fungible_store_primary_store">primary_fungible_store::primary_store</a>(from, asset);
    <a href="fungible_asset.md#0x1_fungible_asset_burn_from">fungible_asset::burn_from</a>(burn_ref, from_wallet, amount);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_freeze_account"></a>

## Function `freeze_account`

Freeze an account so it cannot transfer or receive fungible assets.


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_freeze_account">freeze_account</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="account.md#0x1_account">account</a>: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_freeze_account">freeze_account</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="account.md#0x1_account">account</a>: <b>address</b>) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> {
    <b>let</b> asset = <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>();
    <b>let</b> transfer_ref = &<a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(admin, asset).transfer_ref;
    <b>let</b> wallet = <a href="primary_fungible_store.md#0x1_primary_fungible_store_ensure_primary_store_exists">primary_fungible_store::ensure_primary_store_exists</a>(<a href="account.md#0x1_account">account</a>, asset);
    <a href="fungible_asset.md#0x1_fungible_asset_set_frozen_flag">fungible_asset::set_frozen_flag</a>(transfer_ref, wallet, <b>true</b>);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_unfreeze_account"></a>

## Function `unfreeze_account`

Unfreeze an account so it can transfer or receive fungible assets.


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_unfreeze_account">unfreeze_account</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="account.md#0x1_account">account</a>: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_unfreeze_account">unfreeze_account</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="account.md#0x1_account">account</a>: <b>address</b>) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> {
    <b>let</b> asset = <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>();
    <b>let</b> transfer_ref = &<a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(admin, asset).transfer_ref;
    <b>let</b> wallet = <a href="primary_fungible_store.md#0x1_primary_fungible_store_ensure_primary_store_exists">primary_fungible_store::ensure_primary_store_exists</a>(<a href="account.md#0x1_account">account</a>, asset);
    <a href="fungible_asset.md#0x1_fungible_asset_set_frozen_flag">fungible_asset::set_frozen_flag</a>(transfer_ref, wallet, <b>false</b>);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_set_pause"></a>

## Function `set_pause`

Pause or unpause the transfer of FA coin. This checks that the caller is the pauser.


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_set_pause">set_pause</a>(pauser: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, paused: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_set_pause">set_pause</a>(pauser: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, paused: bool) <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> {
    <b>let</b> asset = <a href="usdt_coin.md#0x1_usdt_coin_get_metadata">get_metadata</a>();
    <b>assert</b>!(<a href="object.md#0x1_object_is_owner">object::is_owner</a>(asset, <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(pauser)), <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_permission_denied">error::permission_denied</a>(<a href="usdt_coin.md#0x1_usdt_coin_ENOT_OWNER">ENOT_OWNER</a>));
    <b>let</b> state = <b>borrow_global_mut</b>&lt;<a href="usdt_coin.md#0x1_usdt_coin_State">State</a>&gt;(<a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&@cedra_framework, <a href="usdt_coin.md#0x1_usdt_coin_ASSET_SYMBOL">ASSET_SYMBOL</a>));
    <b>if</b> (state.paused == paused) { <b>return</b> };
    state.paused = paused;
}
</code></pre>



</details>

<a id="0x1_usdt_coin_assert_not_paused"></a>

## Function `assert_not_paused`

Assert that the FA coin is not paused.
OPTIONAL


<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_assert_not_paused">assert_not_paused</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_assert_not_paused">assert_not_paused</a>() <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_State">State</a> {
    <b>let</b> state = <b>borrow_global</b>&lt;<a href="usdt_coin.md#0x1_usdt_coin_State">State</a>&gt;(<a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&@cedra_framework, <a href="usdt_coin.md#0x1_usdt_coin_ASSET_SYMBOL">ASSET_SYMBOL</a>));
    <b>assert</b>!(!state.paused, <a href="usdt_coin.md#0x1_usdt_coin_EPAUSED">EPAUSED</a>);
}
</code></pre>



</details>

<a id="0x1_usdt_coin_authorized_borrow_refs"></a>

## Function `authorized_borrow_refs`

Borrow the immutable reference of the refs of <code>metadata</code>.
This validates that the signer is the metadata object's owner.


<pre><code><b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(owner: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, asset: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;): &<a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">usdt_coin::ManagedFungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>inline <b>fun</b> <a href="usdt_coin.md#0x1_usdt_coin_authorized_borrow_refs">authorized_borrow_refs</a>(
    owner: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    asset: Object&lt;Metadata&gt;,
): &<a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> <b>acquires</b> <a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a> {
    <b>assert</b>!(<a href="object.md#0x1_object_is_owner">object::is_owner</a>(asset, <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(owner)), <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error_permission_denied">error::permission_denied</a>(<a href="usdt_coin.md#0x1_usdt_coin_ENOT_OWNER">ENOT_OWNER</a>));
    <b>borrow_global</b>&lt;<a href="usdt_coin.md#0x1_usdt_coin_ManagedFungibleAsset">ManagedFungibleAsset</a>&gt;(<a href="object.md#0x1_object_object_address">object::object_address</a>(&asset))
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
