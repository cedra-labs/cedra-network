script {
    use cedra_framework::cedra_governance;
    use cedra_framework::staking_config;
    use cedra_std::fixed_point64;

    fun main(core_resources: &signer) {
        let framework_signer = cedra_governance::get_signer_testnet_only(core_resources, @cedra_framework);
        staking_config::update_rewards_config(
            &framework_signer,
            fixed_point64::create_from_rational(1, 100),
            fixed_point64::create_from_rational(3, 1000),
            365 * 24 * 60 * 60,
            fixed_point64::create_from_rational(50, 100),
        );
        cedra_governance::force_end_epoch(&framework_signer);
    }
}
