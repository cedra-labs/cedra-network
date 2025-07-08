script {
    use cedra_framework::cedra_governance;
    use cedra_framework::coin;
    use cedra_framework::cedra_coin::CedraCoin;
    use cedra_framework::staking_config;

    fun main(proposal_id: u64) {
        let framework_signer = cedra_governance::resolve(proposal_id, @cedra_framework);
        let one_cedra_coin_with_decimals = 10 ** (coin::decimals<CedraCoin>() as u64);
        // Change min to 1000 and max to 1M Cedra coins.
        let new_min_stake = 1000 * one_cedra_coin_with_decimals;
        let new_max_stake = 1000000 * one_cedra_coin_with_decimals;
        staking_config::update_required_stake(&framework_signer, new_min_stake, new_max_stake);
    }
}
