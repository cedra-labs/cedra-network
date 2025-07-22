// Script hash: 4d46f743 
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
            x"22b7bbe49265ef1a747a12efb03e9ef0f8ead35aa2b57e1339f5140f024f68ec",);
        let enabled_blob: vector<u64> = vector[
            95,
        ];

        let disabled_blob: vector<u64> = vector[

        ];

        features::change_feature_flags_for_next_epoch(&framework_signer, enabled_blob, disabled_blob);
        cedra_governance::reconfigure(&framework_signer);
    }
}
