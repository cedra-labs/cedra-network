spec cedra_framework::execution_config {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    /// Ensure the caller is admin
    /// When setting now time must be later than last_reconfiguration_time.
    spec set(account: &signer, config: vector<u8>) {
        use cedra_framework::timestamp;
        use std::signer;
        use std::features;
        use cedra_framework::chain_status;
        use cedra_framework::staking_config;
        use cedra_framework::cedra_coin;

        // TODO: set because of timeout (property proved)
        pragma verify_duration_estimate = 600;
        let addr = signer::address_of(account);
        requires chain_status::is_genesis();
        requires exists<staking_config::StakingRewardsConfig>(@cedra_framework);
        requires len(config) > 0;
        include features::spec_periodical_reward_rate_decrease_enabled() ==> staking_config::StakingRewardsConfigEnabledRequirement;
        include cedra_coin::ExistsCedraCoin;
        requires system_addresses::is_cedra_framework_address(addr);
        requires timestamp::spec_now_microseconds() >= reconfiguration::last_reconfiguration_time();

        ensures exists<ExecutionConfig>(@cedra_framework);
    }

    spec set_for_next_epoch(account: &signer, config: vector<u8>) {
        include config_buffer::SetForNextEpochAbortsIf;
    }

    spec on_new_epoch(framework: &signer) {
        requires @cedra_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<ExecutionConfig>;
        aborts_if false;
    }
}
