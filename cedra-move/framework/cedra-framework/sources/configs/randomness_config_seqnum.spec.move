spec cedra_framework::randomness_config_seqnum {
    spec on_new_epoch(framework: &signer) {
        requires @cedra_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<RandomnessConfigSeqNum>;
        aborts_if false;
    }
}
