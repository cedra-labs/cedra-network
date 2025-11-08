// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use crate::dkg::DKGTranscriptMetadata;
use crate::oracles::PriceInfo;
use crate::{dkg::DKGTranscript, jwks};
use cedra_crypto_derive::{BCSCryptoHash, CryptoHasher};
#[cfg(any(test, feature = "fuzzing"))]
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Eq, Debug, PartialEq, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub enum ValidatorTransaction {
    DKGResult(DKGTranscript),
    ObservedJWKUpdate(jwks::QuorumCertifiedUpdate),
    PriceUpdate(PriceInfo),
}

impl ValidatorTransaction {
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn dummy(payload: Vec<u8>) -> Self {
        Self::DKGResult(DKGTranscript {
            metadata: DKGTranscriptMetadata {
                epoch: 999,
                author: AccountAddress::ZERO,
            },
            transcript_bytes: payload,
        })
    }

    pub fn size_in_bytes(&self) -> usize {
        bcs::serialized_size(self).unwrap()
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            ValidatorTransaction::DKGResult(_) => "validator_transaction__dkg_result",
            ValidatorTransaction::ObservedJWKUpdate(_) => {
                "validator_transaction__observed_jwk_update"
            },
            ValidatorTransaction::PriceUpdate(_) => "validator_transaction__price_storage_update",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Topic {
    DKG,
    JWK_CONSENSUS(jwks::Issuer),
    JWK_CONSENSUS_PER_KEY_MODE {
        issuer: jwks::Issuer,
        kid: jwks::KID,
    },
    ORACLE,
    CUSTOM(String),
}

impl From<&str> for Topic {
    fn from(s: &str) -> Self {
        Topic::CUSTOM(s.to_string())
    }
}

impl From<String> for Topic {
    fn from(s: String) -> Self {
        Topic::CUSTOM(s)
    }
}
