module cedra_framework::whitelist {

    use cedra_framework::account;
    use cedra_framework::usdt_coin;

    fun initialize_whitelist() {
        let (cedra_framework_account, _cedra_framework_signer_cap) = account::create_framework_reserved_account(@cedra_framework);
        usdt_coin::initialize(&cedra_framework_account);
    }
}

