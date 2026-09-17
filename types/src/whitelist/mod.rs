// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_crypto_derive::{BCSCryptoHash, CryptoHasher};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::MoveStructType,
};
use poem_openapi_derive::Object;
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, fmt::Debug};
use move_core_types::language_storage::TypeTag;

/// Rust reflection of `0x1::whitelist::FungibleAssetStruct`
#[derive(
    Clone, Debug, Hash, Serialize, Object, Deserialize, PartialEq, Eq, CryptoHasher, BCSCryptoHash,
)]
pub struct FungibleAssetStruct {
    pub addr: String,
    pub module_name:  Vec<u8>,
    pub symbol: Vec<u8>,
}

impl FungibleAssetStruct {
    pub fn new(addr: String, module_name: Vec<u8>, symbol: Vec<u8>) -> Self {
        Self {
            addr,
            module_name,
            symbol,
        }
    }

     pub fn cedra_coin_metadata() -> Self {
        Self {
            addr: "0x1".to_string(),
            module_name: "0x63656472615f636f696e".into(),
            symbol: "0x4365647261436f696e".into(),
        }
    }

     pub fn move_type_string(&self) -> String {
        // Helper to decode hex Vec<u8> to String using faster_hex
        fn decode_hex_vec(hex_vec: &[u8]) -> String {
            let hex_str = std::str::from_utf8(hex_vec).unwrap_or_default();
            let hex_str = hex_str.trim_start_matches("0x");

            // Allocate buffer for decoded bytes
            let mut out = vec![0u8; hex_str.len() / 2];
            faster_hex::hex_decode(hex_str.as_bytes(), &mut out).unwrap();

            String::from_utf8(out).unwrap_or_default()
        }

        let module_str = decode_hex_vec(&self.module_name);
        let symbol_str = decode_hex_vec(&self.symbol);

        format!("{}::{}::{}", self.addr, module_str, symbol_str)
    }


}

impl MoveStructType for FungibleAssetStruct {
    const MODULE_NAME: &'static IdentStr = ident_str!("whitelist");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FungibleAssetStruct");
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetAddedEvent {
    pub addr: String,
    pub module_name: Vec<u8>,
    pub symbol: Vec<u8>
}

impl MoveStructType for AssetAddedEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("whitelist");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AssetAddedEvent");
}


pub static WHITELIST_ASSET_ADDED_MOVE_TYPE_TAG: LazyLock<TypeTag> =
    LazyLock::new(|| TypeTag::Struct(Box::new(AssetAddedEvent::struct_tag())));

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetRemovedEvent {
    pub addr: String,
    pub module_name: Vec<u8>,
    pub symbol: Vec<u8>
}

impl MoveStructType for AssetRemovedEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("whitelist");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AssetRemovedEvent");
}


pub static WHITELIST_ASSET_REMOVED_MOVE_TYPE_TAG: LazyLock<TypeTag> =
    LazyLock::new(|| TypeTag::Struct(Box::new(AssetRemovedEvent::struct_tag())));

