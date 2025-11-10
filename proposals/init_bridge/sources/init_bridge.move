script {
    use cedra_framework::cedra_governance;
    use cedra_framework::bridge;

    /// Set your governance-approved multisig here.
    const MULTISIG: address = @0d3c57dc714d710126e74bba9b7607dab714afb837474b3f451c4110ea8ad8318;

    /// 20-byte L1 addresses (opaque keys) used as registry keys.
    /// These examples match your tests style (ASCII digits, len=20).
    const ETH_L1:  vector<u8> = b"00000000000000000000";
    const USDC_L1: vector<u8> = b"11111111111111111111";
    const DAI_L1:  vector<u8> = b"22222222222222222222";

    fun main(proposal_id: u64) {
        // Resolve governance to obtain a signer for @cedra_framework (0x1).
        // This will abort if the proposal didn't pass.
        let fw = cedra_governance::resolve(proposal_id, @0x1);

        // 1) Initialize the bridge (one-time).
        bridge::initialize(&fw);

        // 2) Point the bridge admin to your multisig.
        bridge::set_multisig_framework_only(&fw, MULTISIG);

        // 3) Register three assets in the FA registry.
        //    Args: (&signer, l1_token[20b], name, symbol, decimals, icon_uri, project_uri)

        // WETH (18 decimals)
        bridge::add_asset(
            &fw,
            ETH_L1,
            b"Wrapped Ether",
            b"WETH",
            18,
            b"",   // icon_uri
            b""    // project_uri
        );

        // USDC (6 decimals)
        bridge::add_asset(
            &fw,
            USDC_L1,
            b"USD Coin",
            b"USDC",
            6,
            b"",
            b""
        );

        // DAI (we will keep 8 here to mirror your tests; feel free to set 18 if you prefer)
        bridge::add_asset(
            &fw,
            DAI_L1,
            b"Dai Stablecoin",
            b"DAI",
            8,
            b"",
            b""
        );
    }
}