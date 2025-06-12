// Script hash: c2035ec4
script {
    use cedra_framework::cedra_governance;
    use cedra_framework::version;

    fun main(core_resources: &signer) {
        let core_signer = cedra_governance::get_signer_testnet_only(core_resources, @0x1);

        let framework_signer = &core_signer;

        version::set_for_next_epoch(framework_signer, 999);
        cedra_governance::reconfigure(framework_signer);
    }
}
