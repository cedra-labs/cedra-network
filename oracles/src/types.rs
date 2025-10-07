// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_crypto_derive::CryptoHasher;
use cedra_enum_conversion_derive::EnumConversion;
use cedra_reliable_broadcast::RBMessage;
// pub use cedra_types::dkg::DKGTranscript;
use serde::{Deserialize, Serialize};

/// Once DKG starts, a validator should send this message to peers in order to collect DKG transcripts from peers.
#[derive(Clone, Serialize, Deserialize, CryptoHasher, Debug, PartialEq)]
pub struct OraclesRequest {
    dealer_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OraclesResponse {
    dealer_epoch: u64,
}

impl OraclesRequest {
    pub fn new(epoch: u64) -> Self {
        Self {
            dealer_epoch: epoch,
        }
    }
}

/// The Oracles network message.
#[derive(Clone, Serialize, Deserialize, Debug, EnumConversion, PartialEq)]
pub enum OraclesMessage {
    Request(OraclesRequest),
    Response(OraclesResponse),
}

impl RBMessage for OraclesMessage {}
