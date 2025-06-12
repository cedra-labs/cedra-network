module aa::test_functions {
    use cedra_framework::cedra_account;

    /// test function for multi-agent aa.
    public entry fun transfer_to_the_last(a: &signer, b: &signer, c: &signer, d: address) {
        cedra_account::transfer(a, d, 1);
        cedra_account::transfer(b, d, 1);
        cedra_account::transfer(c, d, 1);
    }
}
