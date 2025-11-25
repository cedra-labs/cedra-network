// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_crypto_derive::CryptoHasher;
use cedra_enum_conversion_derive::EnumConversion;
use cedra_reliable_broadcast::RBMessage;
pub use cedra_types::oracles::PriceInfo;
use serde::{Deserialize, Serialize};

/// Once Oracle starts, a validator should send this message to peers in order to collect Oracle PriceInfo from peers.
#[derive(Clone, Serialize, Deserialize, CryptoHasher, Debug, PartialEq)]
pub struct PriceInfoRequest {
    fa_address: Vec<u8>,
}

impl PriceInfoRequest {
    pub fn new(fa_address: Vec<u8>) -> Self {
        Self {
            fa_address,
        }
    }
}

/// The Oracle network message.
#[derive(Clone, Serialize, Deserialize, Debug, EnumConversion, PartialEq)]
pub enum OracleMessage {
    OracleRequest(PriceInfoRequest),
    OracleResponse(PriceInfo),
}

impl OracleMessage {
    pub fn fa_address(&self) -> Vec<u8> {
        match self {
            OracleMessage::OracleRequest(request) => request.fa_address.clone(),
            OracleMessage::OracleResponse(response) => response.fa_address.clone(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            OracleMessage::OracleRequest(_) => "OracleRequest",
            OracleMessage::OracleResponse(_) => "OracleResponse",
        }
    }
}

impl RBMessage for OracleMessage {}
