
<a id="0x1_custom_account"></a>

# Module `0x1::custom_account`

TODO: add tests & recheck batch_transfer logic


-  [Resource `DirectTransferConfig`](#0x1_custom_account_DirectTransferConfig)
-  [Struct `DirectCoinTransferConfigUpdatedEvent`](#0x1_custom_account_DirectCoinTransferConfigUpdatedEvent)
-  [Struct `DirectCoinTransferConfigUpdated`](#0x1_custom_account_DirectCoinTransferConfigUpdated)
-  [Constants](#@Constants_0)
-  [Function `create_account`](#0x1_custom_account_create_account)
-  [Function `transfer`](#0x1_custom_account_transfer)
-  [Function `batch_transfer_fungible_assets`](#0x1_custom_account_batch_transfer_fungible_assets)
-  [Function `transfer_fungible_assets`](#0x1_custom_account_transfer_fungible_assets)
-  [Function `deposit_fungible_assets`](#0x1_custom_account_deposit_fungible_assets)
-  [Function `assert_account_exists`](#0x1_custom_account_assert_account_exists)
-  [Function `assert_account_is_registered_for_fa`](#0x1_custom_account_assert_account_is_registered_for_fa)
-  [Function `set_allow_direct_coin_transfers`](#0x1_custom_account_set_allow_direct_coin_transfers)
-  [Function `can_receive_direct_coin_transfers`](#0x1_custom_account_can_receive_direct_coin_transfers)
-  [Function `fungible_transfer_only`](#0x1_custom_account_fungible_transfer_only)
-  [Function `is_fungible_balance_at_least`](#0x1_custom_account_is_fungible_balance_at_least)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/features.md#0x1_features">0x1::features</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
</code></pre>



<a id="0x1_custom_account_DirectTransferConfig"></a>

## Resource `DirectTransferConfig`

Configuration for whether an account can receive direct transfers of coins that they have not registered.

By default, this is enabled. Users can opt-out by disabling at any time.


<pre><code><b>struct</b> <a href="custom_account.md#0x1_custom_account_DirectTransferConfig">DirectTransferConfig</a> <b>has</b> key
</code></pre>



<a id="0x1_custom_account_DirectCoinTransferConfigUpdatedEvent"></a>

## Struct `DirectCoinTransferConfigUpdatedEvent`

Event emitted when an account's direct coins transfer config is updated.


<pre><code><b>struct</b> <a href="custom_account.md#0x1_custom_account_DirectCoinTransferConfigUpdatedEvent">DirectCoinTransferConfigUpdatedEvent</a> <b>has</b> drop, store
</code></pre>



<a id="0x1_custom_account_DirectCoinTransferConfigUpdated"></a>

## Struct `DirectCoinTransferConfigUpdated`



<pre><code>#[<a href="event.md#0x1_event">event</a>]
<b>struct</b> <a href="custom_account.md#0x1_custom_account_DirectCoinTransferConfigUpdated">DirectCoinTransferConfigUpdated</a> <b>has</b> drop, store
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_custom_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS"></a>

Account opted out of receiving coins that they did not register to receive.


<pre><code><b>const</b> <a href="custom_account.md#0x1_custom_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS">EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS</a>: u64 = 3;
</code></pre>



<a id="0x1_custom_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS"></a>

Account opted out of directly receiving NFT tokens.


<pre><code><b>const</b> <a href="custom_account.md#0x1_custom_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS">EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS</a>: u64 = 4;
</code></pre>



<a id="0x1_custom_account_EACCOUNT_NOT_FOUND"></a>

Account does not exist.


<pre><code><b>const</b> <a href="custom_account.md#0x1_custom_account_EACCOUNT_NOT_FOUND">EACCOUNT_NOT_FOUND</a>: u64 = 1;
</code></pre>



<a id="0x1_custom_account_EACCOUNT_NOT_REGISTERED_FOR_FA"></a>

Account is not registered to receive fungible asset.


<pre><code><b>const</b> <a href="custom_account.md#0x1_custom_account_EACCOUNT_NOT_REGISTERED_FOR_FA">EACCOUNT_NOT_REGISTERED_FOR_FA</a>: u64 = 2;
</code></pre>



<a id="0x1_custom_account_EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH"></a>

The lengths of the recipients and amounts lists don't match.


<pre><code><b>const</b> <a href="custom_account.md#0x1_custom_account_EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH">EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH</a>: u64 = 5;
</code></pre>



<a id="0x1_custom_account_create_account"></a>

## Function `create_account`

Basic account creation methods.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_account.md#0x1_custom_account_create_account">create_account</a>(auth_key: <b>address</b>, fa_address: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_transfer"></a>

## Function `transfer`

Convenient function to transfer FA to a recipient account that might not exist.
This would create the recipient account first, which also registers it to receive FA, before transferring.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_account.md#0x1_custom_account_transfer">transfer</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u64, fa_address: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_batch_transfer_fungible_assets"></a>

## Function `batch_transfer_fungible_assets`



<pre><code><b>public</b> entry <b>fun</b> <a href="custom_account.md#0x1_custom_account_batch_transfer_fungible_assets">batch_transfer_fungible_assets</a>(from: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;, recipients: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, amounts: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u64&gt;, fa_address: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_transfer_fungible_assets"></a>

## Function `transfer_fungible_assets`

Convenient function to deposit fungible asset into a recipient account that might not exist.
This would create the recipient account first to receive the fungible assets.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_account.md#0x1_custom_account_transfer_fungible_assets">transfer_fungible_assets</a>(from: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;, <b>to</b>: <b>address</b>, amount: u64, fa_address: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_deposit_fungible_assets"></a>

## Function `deposit_fungible_assets`

Convenient function to deposit fungible asset into a recipient account that might not exist.
This would create the recipient account first to receive the fungible assets.


<pre><code><b>public</b> <b>fun</b> <a href="custom_account.md#0x1_custom_account_deposit_fungible_assets">deposit_fungible_assets</a>(<b>to</b>: <b>address</b>, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>, fa_address: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_assert_account_exists"></a>

## Function `assert_account_exists`



<pre><code><b>public</b> <b>fun</b> <a href="custom_account.md#0x1_custom_account_assert_account_exists">assert_account_exists</a>(addr: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_assert_account_is_registered_for_fa"></a>

## Function `assert_account_is_registered_for_fa`



<pre><code><b>public</b> <b>fun</b> <a href="custom_account.md#0x1_custom_account_assert_account_is_registered_for_fa">assert_account_is_registered_for_fa</a>(addr: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_set_allow_direct_coin_transfers"></a>

## Function `set_allow_direct_coin_transfers`

Set whether <code><a href="account.md#0x1_account">account</a></code> can receive direct transfers of coins that they have not explicitly registered to receive.


<pre><code><b>public</b> entry <b>fun</b> <a href="custom_account.md#0x1_custom_account_set_allow_direct_coin_transfers">set_allow_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, allow: bool)
</code></pre>



<a id="0x1_custom_account_can_receive_direct_coin_transfers"></a>

## Function `can_receive_direct_coin_transfers`

Return true if <code><a href="account.md#0x1_account">account</a></code> can receive direct transfers of coins that they have not explicitly registered to
receive.

By default, this returns true if an account has not explicitly set whether the can receive direct transfers.


<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="custom_account.md#0x1_custom_account_can_receive_direct_coin_transfers">can_receive_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>



<a id="0x1_custom_account_fungible_transfer_only"></a>

## Function `fungible_transfer_only`

Cedra Primary Fungible Store specific specialized functions,
Utilized internally once migration of Cedra to FungibleAsset is complete.
Convenient function to transfer Cedra to a recipient account that might not exist.
This would create the recipient Cedra PFS first, which also registers it to receive Cedra, before transferring.
TODO: once migration is complete, rename to just "transfer_only" and make it an entry function (for cheapest way
to transfer Cedra) - if we want to allow Cedra PFS without account itself


<pre><code><b>public</b>(<b>friend</b>) entry <b>fun</b> <a href="custom_account.md#0x1_custom_account_fungible_transfer_only">fungible_transfer_only</a>(from: <b>address</b>, <b>to</b>: <b>address</b>, amount: u64, fa_address: <b>address</b>)
</code></pre>



<a id="0x1_custom_account_is_fungible_balance_at_least"></a>

## Function `is_fungible_balance_at_least`

Is balance from Cedra Primary FungibleStore at least the given amount


<pre><code><b>public</b> <b>fun</b> <a href="custom_account.md#0x1_custom_account_is_fungible_balance_at_least">is_fungible_balance_at_least</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, amount: u64, fa_address: <b>address</b>): bool
</code></pre>
