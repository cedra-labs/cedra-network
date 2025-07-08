script {
    use cedra_framework::cedra_governance;
    use cedra_framework::staking_config;

    fun main(proposal_id: u64) {
        let framework_signer = cedra_governance::resolve(proposal_id, @cedra_framework);
        // Update voting power increase limit to 10%.
        staking_config::update_voting_power_increase_limit(&framework_signer, 10);
    }
}
