// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_crypto_derive::{BCSCryptoHash, CryptoHasher};
use move_core_types::account_address::AccountAddress;
use move_core_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveStructType,
};

use once_cell::sync::Lazy;
use poem_openapi_derive::Object;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

// DEFAULT_DECIMALS represents decimals value that helps convert price value from f64 to u64.
const DEFAULT_DECIMALS: u8 = 8;

/// Rust reflection of `0x1::price_storage::PriceInfo`.
#[derive(Clone, Serialize, Object, Deserialize, PartialEq, Eq, CryptoHasher, BCSCryptoHash)]
pub struct PriceInfo {
    /// Address of the fungible asset (FA).
    pub fa_address: String,
    /// Scaled price value (price * 10^decimals).
    pub price: u64,
    /// Number of decimals used for scaling.
    pub decimals: u8,
}

impl PriceInfo {
    pub fn new(fa_address: String, price: u64) -> Self {
        Self {
            fa_address,
            price,
            decimals: DEFAULT_DECIMALS,
        }
    }

    pub fn get_price(&self) -> u64 {
        self.price.clone()
    }

    pub fn get_decimals(&self) -> u8 {
        self.decimals.clone()
    }
}

impl Debug for PriceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PriceInfo {{ fa_address: {}, price: {}, decimals: {} }}",
            self.fa_address, self.price, self.decimals
        )
    }
}

impl MoveStructType for PriceInfo {
    const MODULE_NAME: &'static IdentStr = ident_str!("price_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PriceInfo");
}

pub static PRICE_INFO_TYPE_TAG: Lazy<TypeTag> =
    Lazy::new(|| TypeTag::Struct(Box::new(PriceInfo::struct_tag())));

/// Rust reflection of `0x1::price_storage::PriceStorage`.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PriceStorage {
    pub prices: Vec<PriceInfo>,
}

impl MoveStructType for PriceStorage {
    const MODULE_NAME: &'static IdentStr = ident_str!("price_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PriceStorage");
}

pub static PRICE_STORAGE_TYPE_TAG: Lazy<TypeTag> =
    Lazy::new(|| TypeTag::Struct(Box::new(PriceStorage::struct_tag())));

/// Errors matching the Move module abort codes.
pub const EPRICE_NOT_FOUND: u64 = 1;

/// This represents a validator transaction to update or remove a price.
/// Equivalent to calling `set_price` or `remove_price` in Move.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, CryptoHasher, BCSCryptoHash)]
pub enum OraclePriceTransaction {
    /// Add or update price
    SetPrice(PriceInfo),

    /// Remove price by FA address
    RemovePrice(move_core_types::account_address::AccountAddress),
}

impl OraclePriceTransaction {
    pub fn type_name(&self) -> &'static str {
        match self {
            OraclePriceTransaction::SetPrice(_) => "oracle_price_transaction__set_price",
            OraclePriceTransaction::RemovePrice(_) => "oracle_price_transaction__remove_price",
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        bcs::serialized_size(self).unwrap_or(0)
    }
}

impl PriceInfo {
    pub fn to_move_address(&self) -> AccountAddress {
        AccountAddress::from_hex_literal(&self.fa_address).expect("invalid hex address")
    }
}
