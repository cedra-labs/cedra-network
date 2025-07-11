// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{common::Author, sync_info::SyncInfo, vote::Vote};
use anyhow::ensure;
use cedra_crypto::HashValue;
use cedra_types::validator_verifier::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// VoteMsg is the struct that is ultimately sent by the voter in response for
/// receiving a proposal.
/// VoteMsg carries the `LedgerInfo` of a block that is going to be committed in case this vote
/// is gathers QuorumCertificate (see the detailed explanation in the comments of `LedgerInfo`).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct VoteMsg {
    /// The container for the vote (VoteData, LedgerInfo, Signature)
    vote: Vote,
    /// Sync info carries information about highest QC, TC and LedgerInfo
    sync_info: SyncInfo,
}

impl Display for VoteMsg {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "VoteMsg: [{}], SyncInfo: [{}]",
            self.vote, self.sync_info
        )
    }
}

impl VoteMsg {
    pub fn new(vote: Vote, sync_info: SyncInfo) -> Self {
        Self { vote, sync_info }
    }

    /// Container for actual voting material
    pub fn vote(&self) -> &Vote {
        &self.vote
    }

    /// SyncInfo of the given vote message
    pub fn sync_info(&self) -> &SyncInfo {
        &self.sync_info
    }

    pub fn epoch(&self) -> u64 {
        self.vote.epoch()
    }

    pub fn proposed_block_id(&self) -> HashValue {
        self.vote.vote_data().proposed().id()
    }

    pub fn verify(&self, sender: Author, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        ensure!(
            self.vote().author() == sender,
            "Vote author {:?} is different from the sender {:?}",
            self.vote().author(),
            sender,
        );
        ensure!(
            self.vote().epoch() == self.sync_info.epoch(),
            "VoteMsg has different epoch"
        );
        ensure!(
            self.vote().vote_data().proposed().round() > self.sync_info.highest_round(),
            "Vote Round should be higher than SyncInfo"
        );
        if let Some((timeout, _)) = self.vote().two_chain_timeout() {
            ensure!(
                timeout.hqc_round() <= self.sync_info.highest_certified_round(),
                "2-chain Timeout hqc should be less or equal than the sync info hqc"
            );
        }
        // We're not verifying SyncInfo here yet: we are going to verify it only in case we need
        // it. This way we avoid verifying O(n) SyncInfo messages while aggregating the votes
        // (O(n^2) signature verifications).
        self.vote().verify(validator)
    }
}
