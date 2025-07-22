module cedra_framework::usdt {
    use cedra_framework::custom_fungible_asset;
    use std::string::utf8;

    fun init_module(admin: &signer) {
        custom_fungible_asset::initialize(
            admin,
            b"USDT",                                 // ASSET_SYMBOL: vector<u8>
            utf8(b"Tether USD"),                     // ASSET_NAME: String
            6,                                       // DECIMALS: u8
            utf8(b"https://example.com/usdt.png"),   // ICON_URL: String
            utf8(b"https://tether.to"),              // PROJECT_URL: String
            utf8(b"usdt")                            // MODULE_NAME: String
        );
    }
}
