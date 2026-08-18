// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::move_utils::as_move_value::AsMoveValue;
use super::OnChainConfig;
use cedra_crypto::HashValue;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::ModuleId,
    value::{MoveStruct, MoveValue},
};
use serde::{Deserialize, Serialize};

/// Default oracle package address (resource-account style). Used when a network
/// does not override `oracle_address` in genesis layout.
pub fn default_oracle_address() -> AccountAddress {
    AccountAddress::from_hex_literal(
        "0x108c56518936177dbd434b82b5e0ee287affeba5d702fa0d27348e16c77bda4c",
    )
    .expect("valid default oracle address")
}

/// On-chain `0x1::oracle_config::OracleConfig`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OracleConfig {
    pub addr: AccountAddress,
}

impl OnChainConfig for OracleConfig {
    const MODULE_IDENTIFIER: &'static str = "oracle_config";
    const TYPE_IDENTIFIER: &'static str = "OracleConfig";
}

/// BCS layout of `oracle::i64::I64`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OracleI64 {
    pub negative: bool,
    pub magnitude: u64,
}

/// BCS layout of `oracle::price::Price`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OraclePrice {
    pub price: OracleI64,
    pub conf: u64,
    pub expo: OracleI64,
    pub timestamp: u64,
}

/// BCS layout of `0x1::price_storage::OracleQuote`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OracleQuote {
    pub price_negative: bool,
    pub price_magnitude: u64,
    pub conf: u64,
    pub expo_negative: bool,
    pub expo_magnitude: u64,
    pub timestamp: u64,
}

impl From<&OraclePrice> for OracleQuote {
    fn from(price: &OraclePrice) -> Self {
        Self {
            price_negative: price.price.negative,
            price_magnitude: price.price.magnitude,
            conf: price.conf,
            expo_negative: price.expo.negative,
            expo_magnitude: price.expo.magnitude,
            timestamp: price.timestamp,
        }
    }
}

impl AsMoveValue for OracleQuote {
    fn as_move_value(&self) -> MoveValue {
        MoveValue::Struct(MoveStruct::Runtime(vec![
            self.price_negative.as_move_value(),
            self.price_magnitude.as_move_value(),
            self.conf.as_move_value(),
            self.expo_negative.as_move_value(),
            self.expo_magnitude.as_move_value(),
            self.timestamp.as_move_value(),
        ]))
    }
}

pub const CEDRA_FEED_ADDRESS: &[u8] = b"0x1";
pub const CEDRA_FEED_SYMBOL: &[u8] = b"Cedra";
pub const ORACLE_MODULE_NAME: &IdentStr = ident_str!("oracle");
pub const GET_PRICE_BY_FEED_ID: &IdentStr = ident_str!("get_price_by_feed_id");

pub fn oracle_module_id(oracle_addr: AccountAddress) -> ModuleId {
    ModuleId::new(oracle_addr, ORACLE_MODULE_NAME.to_owned())
}

/// `0x` + 64-char lowercase hex. Matches Move `address_to_hex`.
pub fn fa_feed_address_bytes(addr: AccountAddress) -> Vec<u8> {
    let mut out = b"0x".to_vec();
    out.extend_from_slice(addr.to_canonical_string().as_bytes());
    out
}

/// Matches Move / Go `NewPriceIdentifier`: sha3_256(address_bytes || symbol).
pub fn new_price_feed_id(address_bytes: &[u8], symbol: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(address_bytes.len() + symbol.len());
    data.extend_from_slice(address_bytes);
    data.extend_from_slice(symbol);
    HashValue::sha3_256_of(&data).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cedra_feed_id() {
        let feed_id = new_price_feed_id(CEDRA_FEED_ADDRESS, CEDRA_FEED_SYMBOL);
        assert_eq!(
            hex::encode(feed_id),
            "ab5cfdf4863d0717a8ebe5608f54f508e73b620c4d3855fbbacf8a9dd32c0d13"
        );
    }
}
