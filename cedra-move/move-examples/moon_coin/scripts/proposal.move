script {
    use cedra_framework::cedra_governance;

    fun main(proposal_id: u64) {
        let framework = cedra_governance::resolve_multi_step_proposal(
            proposal_id,
            @0x1,
            vector[118u8,101u8,99u8,116u8,111u8,114u8,58u8,58u8,101u8,109u8,112u8,116u8,121u8,60u8,117u8,56u8,62u8,40u8,41u8,]
        );

        cedra_governance::reconfigure(&framework);
    }
}
