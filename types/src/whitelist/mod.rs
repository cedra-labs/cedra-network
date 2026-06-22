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
use std::{fmt::Debug, sync::LazyLock};
use move_core_types::language_storage::TypeTag;

pub const CEDRA_COIN_TYPE_TAG: &str = "0x1::cedra_coin::CedraCoin";
const STABLECOIN_MODULE: &str = "stablecoin";
const CEDRA_COIN_MODULE: &str = "cedra_coin";

/// Rust reflection of `0x1::whitelist::WhitelistAsset`
#[derive(
    Clone, Debug, Hash, Serialize, Object, Deserialize, PartialEq, Eq, CryptoHasher, BCSCryptoHash,
)]
pub struct FungibleAssetStruct {
    pub addr: String,
    pub symbol: Vec<u8>,
    /// Populated when reading the legacy on-chain registry; preserves oracle price keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_module_name: Option<Vec<u8>>,
}

impl FungibleAssetStruct {
    pub fn new(addr: String, symbol: Vec<u8>) -> Self {
        Self {
            addr,
            symbol,
            legacy_module_name: None,
        }
    }

    pub fn from_legacy(addr: String, module_name: Vec<u8>, symbol: Vec<u8>) -> Self {
        LegacyWhitelistAsset {
            addr,
            module_name,
            symbol,
        }
        .into_canonical()
    }

    pub fn cedra_coin_default() -> Self {
        Self {
            addr: "0x1".to_string(),
            symbol: b"CedraCoin".to_vec(),
            legacy_module_name: None,
        }
    }

    #[deprecated(note = "use cedra_coin_default instead")]
    pub fn cedra_coin_metadata() -> Self {
        Self::cedra_coin_default()
    }

    pub fn is_cedra_coin(&self) -> bool {
        self.addr == "0x1" && Self::bytes_to_utf8(&self.symbol) == "CedraCoin"
    }

    /// Returns the FA type tag used for oracle pricing and fee payment.
    pub fn move_type_string(&self) -> String {
        if self.is_cedra_coin() {
            return CEDRA_COIN_TYPE_TAG.to_string();
        }

        if let Some(module_name) = &self.legacy_module_name {
            let module_str = Self::bytes_to_utf8(module_name);
            if module_str != STABLECOIN_MODULE {
                return format!(
                    "{}::{}::{}",
                    self.addr,
                    module_str,
                    Self::bytes_to_utf8(&self.symbol)
                );
            }
        }

        format!(
            "{}::{}::{}",
            self.addr,
            STABLECOIN_MODULE,
            Self::bytes_to_utf8(&self.symbol)
        )
    }

    fn bytes_to_utf8(bytes: &[u8]) -> String {
        if let Ok(value) = std::str::from_utf8(bytes) {
            return value.to_string();
        }

        let hex_str = std::str::from_utf8(bytes).unwrap_or_default();
        let hex_str = hex_str.trim_start_matches("0x");
        if hex_str.len() % 2 != 0 {
            return String::new();
        }

        let mut out = vec![0u8; hex_str.len() / 2];
        if faster_hex::hex_decode(hex_str.as_bytes(), &mut out).is_err() {
            return String::new();
        }

        String::from_utf8(out).unwrap_or_default()
    }
}

impl MoveStructType for FungibleAssetStruct {
    const MODULE_NAME: &'static IdentStr = ident_str!("whitelist");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FungibleAssetStruct");
}

/// Legacy on-chain layout before migration (`0x1::whitelist::FungibleAssetStruct`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyWhitelistAsset {
    pub addr: String,
    pub module_name: Vec<u8>,
    pub symbol: Vec<u8>,
}

impl LegacyWhitelistAsset {
    pub fn into_canonical(self) -> FungibleAssetStruct {
        let module_str = FungibleAssetStruct::bytes_to_utf8(&self.module_name);
        let legacy_module_name = if self.addr == "0x1"
            && FungibleAssetStruct::bytes_to_utf8(&self.symbol) == "CedraCoin"
        {
            None
        } else if module_str == STABLECOIN_MODULE || module_str == CEDRA_COIN_MODULE {
            None
        } else {
            Some(self.module_name)
        };

        FungibleAssetStruct {
            addr: self.addr,
            symbol: self.symbol,
            legacy_module_name,
        }
    }
}

impl MoveStructType for LegacyWhitelistAsset {
    const MODULE_NAME: &'static IdentStr = ident_str!("whitelist");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FungibleAssetStruct");
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetAddedEvent {
    pub addr: String,
    pub module_name: Vec<u8>,
    pub symbol: Vec<u8>,
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
    pub symbol: Vec<u8>,
}

impl MoveStructType for AssetRemovedEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("whitelist");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AssetRemovedEvent");
}

pub static WHITELIST_ASSET_REMOVED_MOVE_TYPE_TAG: LazyLock<TypeTag> =
    LazyLock::new(|| TypeTag::Struct(Box::new(AssetRemovedEvent::struct_tag())));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cedra_coin_move_type_string() {
        let asset = FungibleAssetStruct::cedra_coin_default();
        assert!(asset.is_cedra_coin());
        assert_eq!(asset.move_type_string(), CEDRA_COIN_TYPE_TAG);
    }

    #[test]
    fn stablecoin_move_type_string() {
        let asset = FungibleAssetStruct::new("0xabc".to_string(), b"USDC".to_vec());
        assert_eq!(asset.move_type_string(), "0xabc::stablecoin::USDC");
    }

    #[test]
    fn legacy_oracle_type_tag_preserved() {
        let legacy = FungibleAssetStruct::from_legacy(
            "0xabc".to_string(),
            b"custom_module".to_vec(),
            b"USDC".to_vec(),
        );
        assert_eq!(legacy.move_type_string(), "0xabc::custom_module::USDC");
    }

    #[test]
    fn legacy_stablecoin_module_uses_canonical_tag() {
        let legacy = LegacyWhitelistAsset {
            addr: "0xabc".to_string(),
            module_name: b"stablecoin".to_vec(),
            symbol: b"USDC".to_vec(),
        }
        .into_canonical();
        assert_eq!(legacy.move_type_string(), "0xabc::stablecoin::USDC");
    }
}
