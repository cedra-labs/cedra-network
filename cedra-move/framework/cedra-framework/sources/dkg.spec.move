spec cedra_framework::dkg {

    spec module {
        use cedra_framework::chain_status;
        invariant [suspendable] chain_status::is_operating() ==> exists<DKGState>(@cedra_framework);
    }

    spec initialize(cedra_framework: &signer) {
        use std::signer;
        let cedra_framework_addr = signer::address_of(cedra_framework);
        aborts_if cedra_framework_addr != @cedra_framework;
    }

    spec start(
        dealer_epoch: u64,
        randomness_config: RandomnessConfig,
        dealer_validator_set: vector<ValidatorConsensusInfo>,
        target_validator_set: vector<ValidatorConsensusInfo>,
    ) {
        aborts_if !exists<DKGState>(@cedra_framework);
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(@cedra_framework);
    }

    spec finish(transcript: vector<u8>) {
        use std::option;
        requires exists<DKGState>(@cedra_framework);
        requires option::is_some(global<DKGState>(@cedra_framework).in_progress);
        aborts_if false;
    }

    spec fun has_incomplete_session(): bool {
        if (exists<DKGState>(@cedra_framework)) {
            option::spec_is_some(global<DKGState>(@cedra_framework).in_progress)
        } else {
            false
        }
    }

    spec try_clear_incomplete_session(fx: &signer) {
        use std::signer;
        let addr = signer::address_of(fx);
        aborts_if addr != @cedra_framework;
    }

    spec incomplete_session(): Option<DKGSessionState> {
        aborts_if false;
    }
}
