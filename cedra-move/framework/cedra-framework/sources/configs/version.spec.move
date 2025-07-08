spec cedra_framework::version {
    /// <high-level-req>
    /// No.: 1
    /// Requirement: During genesis, the Version resource should be initialized with the initial version and stored along
    /// with its capability under the cedra framework account.
    /// Criticality: Medium
    /// Implementation: The initialize function ensures that the signer is the cedra framework account and stores the
    /// Version and SetVersionCapability resources in it.
    /// Enforcement: Formally verified via [high-level-req-1](initialize).
    ///
    /// No.: 2
    /// Requirement: The version should be updateable after initialization, but only by the Cedra framework account and
    /// with an increasing version number.
    /// Criticality: Medium
    /// Implementation: The version number for the blockchain should be updatable whenever necessary. This functionality
    /// is provided by the set_version function which ensures that the new version is greater than the previous one.
    /// Enforcement: Formally verified via [high-level-req-2](set_version).
    /// </high-level-req>
    ///
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec set_version(account: &signer, major: u64) {
        use std::signer;
        use cedra_framework::chain_status;
        use cedra_framework::timestamp;
        use cedra_framework::coin::CoinInfo;
        use cedra_framework::cedra_coin::CedraCoin;
        use cedra_framework::staking_config;
        use cedra_framework::reconfiguration;

        // TODO: set because of timeout (property proved)
        pragma verify_duration_estimate = 120;
        include staking_config::StakingRewardsConfigRequirement;
        requires chain_status::is_genesis();
        requires timestamp::spec_now_microseconds() >= reconfiguration::last_reconfiguration_time();
        requires exists<CoinInfo<CedraCoin>>(@cedra_framework);

        aborts_if !exists<SetVersionCapability>(signer::address_of(account));
        aborts_if !exists<Version>(@cedra_framework);

        let old_major = global<Version>(@cedra_framework).major;
        /// [high-level-req-2]
        aborts_if !(old_major < major);

        ensures global<Version>(@cedra_framework).major == major;
    }

    /// Abort if resource already exists in `@cedra_framwork` when initializing.
    spec initialize(cedra_framework: &signer, initial_version: u64) {
        use std::signer;

        /// [high-level-req-1]
        aborts_if signer::address_of(cedra_framework) != @cedra_framework;
        aborts_if exists<Version>(@cedra_framework);
        aborts_if exists<SetVersionCapability>(@cedra_framework);
        ensures exists<Version>(@cedra_framework);
        ensures exists<SetVersionCapability>(@cedra_framework);
        ensures global<Version>(@cedra_framework) == Version { major: initial_version };
        ensures global<SetVersionCapability>(@cedra_framework) == SetVersionCapability {};
    }

    spec set_for_next_epoch(account: &signer, major: u64) {
        aborts_if !exists<SetVersionCapability>(signer::address_of(account));
        aborts_if !exists<Version>(@cedra_framework);
        aborts_if global<Version>(@cedra_framework).major >= major;
        aborts_if !exists<config_buffer::PendingConfigs>(@cedra_framework);
    }

    spec on_new_epoch(framework: &signer) {
        requires @cedra_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<Version>;
        aborts_if false;
    }

    /// This module turns on `aborts_if_is_strict`, so need to add spec for test function `initialize_for_test`.
    spec initialize_for_test {
        // Don't verify test functions.
        pragma verify = false;
    }
}
