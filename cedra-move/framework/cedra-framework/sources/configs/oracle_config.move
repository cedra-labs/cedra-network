/// On-chain oracle module address used for FA fee pricing.
/// Set at genesis per network and updated only by framework governance.
module cedra_framework::oracle_config {
    use std::error;
    use cedra_framework::chain_status;
    use cedra_framework::config_buffer;
    use cedra_framework::system_addresses;

    friend cedra_framework::genesis;
    friend cedra_framework::reconfiguration_with_dkg;

    /// Oracle address must be non-zero.
    const EINVALID_ORACLE_ADDRESS: u64 = 1;
    /// Oracle config has not been initialized.
    const EORACLE_CONFIG_NOT_FOUND: u64 = 2;

    struct OracleConfig has drop, key, store {
        addr: address,
    }

    public(friend) fun initialize(cedra_framework: &signer, oracle_addr: address) {
        system_addresses::assert_cedra_framework(cedra_framework);
        assert!(oracle_addr != @0x0, error::invalid_argument(EINVALID_ORACLE_ADDRESS));
        move_to(cedra_framework, OracleConfig { addr: oracle_addr });
    }

    /// Genesis-only synchronous update. Prefer `set_for_next_epoch` after genesis.
    public fun set(account: &signer, oracle_addr: address) acquires OracleConfig {
        system_addresses::assert_cedra_framework(account);
        chain_status::assert_genesis();
        assert!(oracle_addr != @0x0, error::invalid_argument(EINVALID_ORACLE_ADDRESS));
        borrow_global_mut<OracleConfig>(@cedra_framework).addr = oracle_addr;
    }

    /// Governance update applied at the next epoch boundary.
    ///
    /// ```
    /// cedra_framework::oracle_config::set_for_next_epoch(&framework_signer, oracle_addr);
    /// cedra_framework::cedra_governance::reconfigure(&framework_signer);
    /// ```
    public fun set_for_next_epoch(account: &signer, oracle_addr: address) {
        system_addresses::assert_cedra_framework(account);
        assert!(oracle_addr != @0x0, error::invalid_argument(EINVALID_ORACLE_ADDRESS));
        config_buffer::upsert(OracleConfig { addr: oracle_addr });
    }

    public(friend) fun on_new_epoch(framework: &signer) acquires OracleConfig {
        system_addresses::assert_cedra_framework(framework);
        if (config_buffer::does_exist<OracleConfig>()) {
            let new_config = config_buffer::extract_v2<OracleConfig>();
            if (exists<OracleConfig>(@cedra_framework)) {
                *borrow_global_mut<OracleConfig>(@cedra_framework) = new_config;
            } else {
                move_to(framework, new_config);
            };
        }
    }

    #[view]
    public fun oracle_address(): address acquires OracleConfig {
        assert!(exists<OracleConfig>(@cedra_framework), error::not_found(EORACLE_CONFIG_NOT_FOUND));
        borrow_global<OracleConfig>(@cedra_framework).addr
    }
}
