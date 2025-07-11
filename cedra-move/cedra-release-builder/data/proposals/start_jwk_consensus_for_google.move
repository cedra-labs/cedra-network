// Start JWK Consensus for Google.
script {
    use cedra_framework::cedra_governance;
    use cedra_framework::jwks;

    fun main(proposal_id: u64) {
        let framework_signer = cedra_governance::resolve_multi_step_proposal(
            proposal_id,
            @0x1,
            {{ script_hash }},
        );

        jwks::upsert_oidc_provider_for_next_epoch(
            &framework_signer,
            b"https://accounts.google.com",
            b"https://accounts.google.com/.well-known/openid-configuration"
        );
        cedra_governance::reconfigure(&framework_signer);
    }
}
