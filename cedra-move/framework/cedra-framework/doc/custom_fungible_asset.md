
<a id="0x1_custom_fungible_asset"></a>

# Module `0x1::custom_fungible_asset`



-  [Resource `ManagedFungibleAsset`](#0x1_custom_fungible_asset_ManagedFungibleAsset)
-  [Resource `State`](#0x1_custom_fungible_asset_State)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_custom_fungible_asset_initialize)
-  [Function `get_metadata`](#0x1_custom_fungible_asset_get_metadata)
-  [Function `deposit`](#0x1_custom_fungible_asset_deposit)
-  [Function `withdraw`](#0x1_custom_fungible_asset_withdraw)
-  [Function `mint`](#0x1_custom_fungible_asset_mint)
-  [Function `transfer`](#0x1_custom_fungible_asset_transfer)
-  [Function `burn`](#0x1_custom_fungible_asset_burn)
-  [Function `freeze_account`](#0x1_custom_fungible_asset_freeze_account)
-  [Function `unfreeze_account`](#0x1_custom_fungible_asset_unfreeze_account)
-  [Function `set_pause`](#0x1_custom_fungible_asset_set_pause)


<pre><code><b>use</b> <a href="dispatchable_fungible_asset.md#0x1_dispatchable_fungible_asset">0x1::dispatchable_fungible_asset</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="function_info.md#0x1_function_info">0x1::function_info</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
</code></pre>



<a id="0x1_custom_fungible_asset_ManagedFungibleAsset"></a>

## Resource `ManagedFungibleAsset`

Hold refs to control the minting, transfer and burning of fungible assets.


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_ManagedFungibleAsset">ManagedFungibleAsset</a> <b>has</b> key
</code></pre>



<a id="0x1_custom_fungible_asset_State"></a>

## Resource `State`

Global state to pause the FA coin.
OPTIONAL


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_State">State</a> <b>has</b> key
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_custom_fungible_asset_ENOT_OWNER"></a>

Only fungible asset metadata owner can make changes.


<pre><code><b>const</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_ENOT_OWNER">ENOT_OWNER</a>: u64 = 1;
</code></pre>



<a id="0x1_custom_fungible_asset_EPAUSED"></a>

The FA coin is paused.


<pre><code><b>const</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_EPAUSED">EPAUSED</a>: u64 = 2;
</code></pre>



<a id="0x1_custom_fungible_asset_initialize"></a>

## Function `initialize`

Initialize metadata object and store the refs.


<pre><code><b>public</b> <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_initialize">initialize</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, name: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, decimals: u8, icon_url: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, project_url: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, module_name: <a href="../../cedra-stdlib/../move-stdlib/doc/string.md#0x1_string_String">string::String</a>)
</code></pre>



<a id="0x1_custom_fungible_asset_get_metadata"></a>

## Function `get_metadata`

Return the address of the managed fungible asset that's created when this module is deployed.


<pre><code><b>public</b> <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_get_metadata">get_metadata</a>(symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;
</code></pre>



<a id="0x1_custom_fungible_asset_deposit"></a>

## Function `deposit`

Deposit function override to ensure that the account is not denylisted and the FA coin is not paused.
OPTIONAL


<pre><code><b>public</b> <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_deposit">deposit</a>&lt;T: key&gt;(store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;T&gt;, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>, transfer_ref: &<a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x1_custom_fungible_asset_withdraw"></a>

## Function `withdraw`

Withdraw function override to ensure that the account is not denylisted and the FA coin is not paused.
OPTIONAL


<pre><code><b>public</b> <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_withdraw">withdraw</a>&lt;T: key&gt;(store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;T&gt;, amount: u64, transfer_ref: &<a href="fungible_asset.md#0x1_fungible_asset_TransferRef">fungible_asset::TransferRef</a>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<a id="0x1_custom_fungible_asset_mint"></a>

## Function `mint`

Mint as the owner of metadata object.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_mint">mint</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <b>to</b>: <b>address</b>, amount: u64, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x1_custom_fungible_asset_transfer"></a>

## Function `transfer`

Transfer as the owner of metadata object ignoring <code>frozen</code> field.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_transfer">transfer</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u64, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x1_custom_fungible_asset_burn"></a>

## Function `burn`

Burn fungible assets as the owner of metadata object.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_burn">burn</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, from: <b>address</b>, amount: u64, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x1_custom_fungible_asset_freeze_account"></a>

## Function `freeze_account`

Freeze an account so it cannot transfer or receive fungible assets.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_freeze_account">freeze_account</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="account.md#0x1_account">account</a>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x1_custom_fungible_asset_unfreeze_account"></a>

## Function `unfreeze_account`

Unfreeze an account so it can transfer or receive fungible assets.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_unfreeze_account">unfreeze_account</a>(admin: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="account.md#0x1_account">account</a>: <b>address</b>, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x1_custom_fungible_asset_set_pause"></a>

## Function `set_pause`

Pause or unpause the transfer of FA coin. This checks that the caller is the pauser.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_fungible_asset.md#0x1_custom_fungible_asset_set_pause">set_pause</a>(pauser: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, paused: bool, symbol: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>
