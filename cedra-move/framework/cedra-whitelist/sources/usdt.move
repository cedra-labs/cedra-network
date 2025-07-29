module admin::usdt {
    use 0x1::custom_fungible_asset;
    use std::string;

    public entry fun init(admin: &signer) {
        custom_fungible_asset::create_fa(
            admin,
            b"USDT",
            string::utf8(b"Tether USD"),
            6,
            string::utf8(b"https://example.com/usdt.png"),
            string::utf8(b"https://tether.to")
        );
    }
}
