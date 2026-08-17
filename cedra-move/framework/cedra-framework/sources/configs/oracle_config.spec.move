spec cedra_framework::oracle_config {
    spec module {
        use cedra_framework::chain_status;
        pragma verify = true;
        pragma aborts_if_is_strict;
        invariant [suspendable] chain_status::is_operating() ==> exists<OracleConfig>(@cedra_framework);
    }

    spec initialize(cedra_framework: &signer, oracle_addr: address) {
        use std::signer;
        let addr = signer::address_of(cedra_framework);
        aborts_if !system_addresses::is_cedra_framework_address(addr);
        aborts_if oracle_addr == @0x0;
        aborts_if exists<OracleConfig>(@cedra_framework);
        ensures global<OracleConfig>(addr).addr == oracle_addr;
    }

    spec set(account: &signer, oracle_addr: address) {
        use std::signer;
        pragma aborts_if_is_partial;
        let addr = signer::address_of(account);
        aborts_if !system_addresses::is_cedra_framework_address(addr);
        aborts_if oracle_addr == @0x0;
        aborts_if !exists<OracleConfig>(@cedra_framework);
        requires chain_status::is_genesis();
        ensures global<OracleConfig>(@cedra_framework).addr == oracle_addr;
    }

    spec set_for_next_epoch(account: &signer, oracle_addr: address) {
        let addr = std::signer::address_of(account);
        aborts_if addr != @cedra_framework;
        aborts_if oracle_addr == @0x0;
        aborts_if !exists<config_buffer::PendingConfigs>(@cedra_framework);
    }

    spec on_new_epoch(framework: &signer) {
        requires @cedra_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<OracleConfig>;
        aborts_if false;
    }

    spec oracle_address(): address {
        aborts_if !exists<OracleConfig>(@cedra_framework);
        ensures result == global<OracleConfig>(@cedra_framework).addr;
    }
}
