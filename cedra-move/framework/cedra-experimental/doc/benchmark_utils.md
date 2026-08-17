
<a id="0x7_benchmark_utils"></a>

# Module `0x7::benchmark_utils`



-  [Function `transfer_and_create_account`](#0x7_benchmark_utils_transfer_and_create_account)


<pre><code><b>use</b> <a href="../../cedra-framework/doc/account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../cedra-framework/doc/cedra_account.md#0x1_cedra_account">0x1::cedra_account</a>;
</code></pre>



<a id="0x7_benchmark_utils_transfer_and_create_account"></a>

## Function `transfer_and_create_account`

Entry function that creates account resource, and funds the account.
This makes sure that transactions later don't need to create an account,
and so actual costs of entry functions can be more precisely measured.


<pre><code>entry <b>fun</b> <a href="benchmark_utils.md#0x7_benchmark_utils_transfer_and_create_account">transfer_and_create_account</a>(source: &<a href="../../cedra-framework/../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <b>to</b>: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="benchmark_utils.md#0x7_benchmark_utils_transfer_and_create_account">transfer_and_create_account</a>(source: &<a href="../../cedra-framework/../cedra-stdlib/../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <b>to</b>: <b>address</b>, amount: u64) {
    <a href="../../cedra-framework/doc/account.md#0x1_account_create_account_if_does_not_exist">account::create_account_if_does_not_exist</a>(<b>to</b>);
    <a href="../../cedra-framework/doc/cedra_account.md#0x1_cedra_account_transfer">cedra_account::transfer</a>(source, <b>to</b>, amount);
}
</code></pre>



</details>


[move-book]: https://cedra.dev/move/book/SUMMARY
