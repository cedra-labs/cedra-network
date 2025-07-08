script {
    use cedra_framework::cedra_governance;
    use cedra_framework::keyless_account;

    fun main(
        core_resources: &signer,
         alpha_g1: vector<u8>,
         beta_g2: vector<u8>,
         gamma_g2: vector<u8>,
         delta_g2: vector<u8>,
         gamma_abc_g1_0: vector<u8>,
         gamma_abc_g1_1: vector<u8>
    ) {
        let vk = keyless_account::new_groth16_verification_key(alpha_g1, beta_g2, gamma_g2, delta_g2, vector[gamma_abc_g1_0, gamma_abc_g1_1]);
        let fx = cedra_governance::get_signer_testnet_only(core_resources, @cedra_framework);
        keyless_account::set_groth16_verification_key_for_next_epoch(&fx, vk);
        // sets the pending Configuration change to the max expiration horizon from above
        cedra_governance::force_end_epoch_test_only(core_resources);
    }
}
