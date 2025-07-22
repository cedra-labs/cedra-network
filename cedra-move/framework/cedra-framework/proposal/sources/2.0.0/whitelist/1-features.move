// Script hash: ef913d31 
// Modifying on-chain feature flags:
// Enabled Features: [FeeV2]
// Disabled Features: []
//
script {
    use cedra_framework::cedra_governance;
    use std::features;

    fun main(proposal_id: u64) {
        let framework_signer = cedra_governance::resolve_multi_step_proposal(
            proposal_id,
            @0x1,
            x"76854b5febe580f0f84719d54630b537d9ca5bc63309e222c0f4cbb710b17faa",);
        let enabled_blob: vector<u64> = vector[
            95,
        ];

        let disabled_blob: vector<u64> = vector[

        ];

        features::change_feature_flags_for_next_epoch(&framework_signer, enabled_blob, disabled_blob);
        cedra_governance::reconfigure(&framework_signer);
    }
}
