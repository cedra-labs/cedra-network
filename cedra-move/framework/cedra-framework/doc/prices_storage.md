
<a id="0x1_price_list"></a>

# Module `0x1::price_list`



-  [Struct `PriceInfo`](#0x1_price_list_PriceInfo)
-  [Resource `PriceStorage`](#0x1_price_list_PriceStorage)
-  [Constants](#@Constants_0)
-  [Function `init`](#0x1_price_list_init)
-  [Function `set_price`](#0x1_price_list_set_price)
-  [Function `remove_price`](#0x1_price_list_remove_price)
-  [Function `get_price`](#0x1_price_list_get_price)


<pre><code><b>use</b> <a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
</code></pre>



<a id="0x1_price_list_PriceInfo"></a>

## Struct `PriceInfo`



<pre><code><b>struct</b> <a href="prices_storage.md#0x1_price_list_PriceInfo">PriceInfo</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>price: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>decimals: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_price_list_PriceStorage"></a>

## Resource `PriceStorage`



<pre><code><b>struct</b> <a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>prices: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="prices_storage.md#0x1_price_list_PriceInfo">price_list::PriceInfo</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_price_list_EPRICE_NOT_FOUND"></a>

Price not founded in storage


<pre><code><b>const</b> <a href="prices_storage.md#0x1_price_list_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>: u64 = 1;
</code></pre>



<a id="0x1_price_list_init"></a>

## Function `init`



<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_init">init</a>(owner: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_init">init</a>(owner: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>move_to</b>(
        owner,
        <a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a> {
            prices: <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="prices_storage.md#0x1_price_list_PriceInfo">PriceInfo</a>&gt;()
        }
    );
}
</code></pre>



</details>

<a id="0x1_price_list_set_price"></a>

## Function `set_price`



<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_set_price">set_price</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa_address: <b>address</b>, price: u64, decimals: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_set_price">set_price</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa_address: <b>address</b>, price: u64, decimals: u8
) <b>acquires</b> <a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a> {
    <b>let</b> store = <b>borrow_global_mut</b>&lt;<a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a>&gt;(<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <b>let</b> len = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&store.prices);
    <b>let</b> i = 0;

    <b>while</b> (i &lt; len) {
        <b>let</b> p_ref = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> store.prices, i);

        <b>if</b> (p_ref.fa_address == fa_address) {
            p_ref.price = price;
            p_ref.decimals = decimals;
            <b>return</b>;
        };
        i = i + 1;
    };

    <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> store.prices,
        <a href="prices_storage.md#0x1_price_list_PriceInfo">PriceInfo</a> { fa_address, price, decimals }
    );
}
</code></pre>



</details>

<a id="0x1_price_list_remove_price"></a>

## Function `remove_price`



<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_remove_price">remove_price</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_remove_price">remove_price</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa_address: <b>address</b>) <b>acquires</b> <a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a> {
    <b>let</b> store = <b>borrow_global_mut</b>&lt;<a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a>&gt;(<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <b>let</b> len = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&store.prices);
    <b>let</b> i = 0;

    <b>while</b> (i &lt; len) {
        <b>let</b> p_ref = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&store.prices, i);
        <b>if</b> (p_ref.fa_address == fa_address) {
            <b>let</b> last = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_pop_back">vector::pop_back</a>(&<b>mut</b> store.prices);
            <b>if</b> (i &lt; <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&store.prices)) {
                <b>let</b> p_mut = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> store.prices, i);
                *p_mut = last;
            };
            <b>return</b>;
        };
        i = i + 1;
    };
    <b>abort</b> <a href="prices_storage.md#0x1_price_list_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>

}
</code></pre>



</details>

<a id="0x1_price_list_get_price"></a>

## Function `get_price`



<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_get_price">get_price</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa_address: <b>address</b>): (u64, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="prices_storage.md#0x1_price_list_get_price">get_price</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa_address: <b>address</b>): (u64, u8) <b>acquires</b> <a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a> {
    <b>let</b> store = <b>borrow_global</b>&lt;<a href="prices_storage.md#0x1_price_list_PriceStorage">PriceStorage</a>&gt;(<a href="../../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <b>let</b> len = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&store.prices);
    <b>let</b> i = 0;

    <b>while</b> (i &lt; len) {
        <b>let</b> p_ref = <a href="../../cedra-stdlib/../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&store.prices, i);
        <b>if</b> (p_ref.fa_address == fa_address) {
            <b>return</b> (p_ref.price, p_ref.decimals);
        };
        i = i + 1;
    };
    <b>abort</b> <a href="prices_storage.md#0x1_price_list_EPRICE_NOT_FOUND">EPRICE_NOT_FOUND</a>

}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
