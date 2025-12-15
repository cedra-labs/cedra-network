// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::move_utils::as_move_value::AsMoveValue;
use cedra_crypto_derive::{BCSCryptoHash, CryptoHasher};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::MoveStructType,
    value::{MoveStruct, MoveValue},
};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(
    Clone, Debug, Hash, Object, Serialize, Deserialize, PartialEq, Eq, CryptoHasher, BCSCryptoHash,
)]

pub struct Prices {
    prices: Vec<PriceInfo>,
}
impl Prices {
    pub fn new(prices: Vec<PriceInfo>) -> Self {
        Self { prices }
    }
}

/// Rust reflection of `0x1::price_storage::PriceInfo`
#[derive(
    Clone, Debug, Object, Hash, Serialize, Deserialize, PartialEq, Eq, CryptoHasher, BCSCryptoHash,
)]
pub struct PriceInfo {
    pub fa_address: String,
    /// Scaled price value (price * 10^decimals)
    pub price: u64,
    /// Number of decimals used for scaling
    pub decimals: u8,
}

impl PriceInfo {
    pub fn new(fa_address: String, price: u64, decimals: u8) -> Self {
        Self {
            fa_address,
            price,
            decimals,
        }
    }
}

impl MoveStructType for PriceInfo {
    const MODULE_NAME: &'static IdentStr = ident_str!("price_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PriceInfo");
}

impl AsMoveValue for PriceInfo {
    fn as_move_value(&self) -> MoveValue {
        MoveValue::Struct(MoveStruct::Runtime(vec![
            self.fa_address.as_move_value(),
            self.price.as_move_value(),
            self.decimals.as_move_value(),
        ]))
    }
}
impl AsMoveValue for Prices {
    fn as_move_value(&self) -> MoveValue {
        MoveValue::Struct(MoveStruct::Runtime(vec![self.prices.as_move_value()]))
    }
}
