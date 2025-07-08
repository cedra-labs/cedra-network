// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::pipeline::signing_phase::CommitSignerProvider;
use cedra_crypto::bls12381;
use cedra_types::validator_signer::ValidatorSigner;
use std::sync::Arc;

pub struct DagCommitSigner {
    signer: Arc<ValidatorSigner>,
}

impl DagCommitSigner {
    pub fn new(signer: Arc<ValidatorSigner>) -> Self {
        Self { signer }
    }
}

impl CommitSignerProvider for DagCommitSigner {
    fn sign_commit_vote(
        &self,
        _ledger_info: cedra_types::ledger_info::LedgerInfoWithSignatures,
        new_ledger_info: cedra_types::ledger_info::LedgerInfo,
    ) -> Result<bls12381::Signature, cedra_safety_rules::Error> {
        let signature = self
            .signer
            .sign(&new_ledger_info)
            .map_err(|err| cedra_safety_rules::Error::SerializationError(err.to_string()))?;

        Ok(signature)
    }
}
