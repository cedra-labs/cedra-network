// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    utility_coin::{CedraCoinType, CoinType},
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// Fee-asset identity: creator address + symbol.
/// On the signed `RawTransaction` this is stored as a `TypeTag` (legacy BCS).
/// API / VM use this struct. Conversion:
/// - Native Cedra: `0x1::cedra_coin::CedraCoin` ↔ `(0x1, "Cedra")`
/// - FA coin: `address::<lowercase(symbol)>::<symbol>` (e.g. `0xc745…::usdct::USDCT`)
/// - Empty: non-struct TypeTag (canonical encode is `bool`)
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct FaAddress {
    pub address: AccountAddress,
    pub symbol: Vec<u8>,
}

impl FaAddress {
    pub const NATIVE_SYMBOL: &'static [u8] = b"Cedra";

    pub fn new(address: AccountAddress, symbol: impl Into<Vec<u8>>) -> Self {
        Self {
            address,
            symbol: symbol.into(),
        }
    }

    /// Empty fee-asset identity. Must stay empty on the wire; filling in native
    /// Cedra would change the signed BCS and fail signature verification.
    pub fn empty() -> Self {
        Self {
            address: AccountAddress::ZERO,
            symbol: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.symbol.is_empty()
    }

    /// Fee v2 applies only to a non-empty, non-native FA coin.
    pub fn use_fee_v2(&self) -> bool {
        !self.is_empty() && !self.is_native_cedra()
    }

    /// Native Cedra coin (`0x1`, `"Cedra"`).
    pub fn native_cedra() -> Self {
        Self {
            address: AccountAddress::ONE,
            symbol: Self::NATIVE_SYMBOL.to_vec(),
        }
    }

    pub fn is_native_cedra(&self) -> bool {
        self.address == AccountAddress::ONE && self.symbol == Self::NATIVE_SYMBOL
    }

    pub fn symbol_str(&self) -> String {
        String::from_utf8_lossy(&self.symbol).into_owned()
    }

    /// Decode the signed TypeTag into address + symbol. Does not rewrite the tag.
    pub fn from_type_tag(tag: &TypeTag) -> Self {
        match tag {
            TypeTag::Struct(s) => {
                if s.module.as_str() == "cedra_coin" && s.name.as_str() == "CedraCoin" {
                    Self::native_cedra()
                } else {
                    Self::new(s.address, s.name.as_str().as_bytes())
                }
            },
            _ => Self::empty(),
        }
    }

    /// Encode address + symbol as the TypeTag stored on `RawTransaction`.
    pub fn to_type_tag(&self) -> TypeTag {
        self.try_to_type_tag()
            .expect("fa_address symbol must be a valid Move identifier")
    }

    pub fn try_to_type_tag(&self) -> anyhow::Result<TypeTag> {
        if self.is_empty() {
            return Ok(TypeTag::Bool);
        }
        if self.is_native_cedra() {
            return Ok(CedraCoinType::type_tag());
        }
        let symbol = self.symbol_str();
        let module = symbol.to_lowercase();
        Ok(TypeTag::Struct(Box::new(StructTag {
            address: self.address,
            module: Identifier::new(module)?,
            name: Identifier::new(symbol)?,
            type_args: vec![],
        })))
    }
}

impl From<FaAddress> for TypeTag {
    fn from(fa: FaAddress) -> Self {
        fa.to_type_tag()
    }
}

impl Default for FaAddress {
    fn default() -> Self {
        Self::empty()
    }
}

impl Display for FaAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }
        write!(f, "{}::{}", self.address, self.symbol_str())
    }
}

impl FromStr for FaAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Ok(Self::empty());
        }
        let parts: Vec<&str> = s.split("::").collect();
        match parts.as_slice() {
            [addr, symbol] => Ok(Self::new(
                AccountAddress::from_str(addr)?,
                symbol.as_bytes(),
            )),
            // Legacy CLI string `address::module::Name`; module is ignored except Cedra.
            [addr, module, name] => {
                let address = AccountAddress::from_str(addr)?;
                if *module == "cedra_coin" && *name == "CedraCoin" {
                    Ok(Self::native_cedra())
                } else {
                    Ok(Self::new(address, name.as_bytes()))
                }
            },
            _ => anyhow::bail!(
                "invalid fa_address '{}': expected address::symbol or address::module::name",
                s
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_cedra_roundtrip_is_cedra_coin_type_tag() {
        let tag = FaAddress::native_cedra().to_type_tag();
        assert_eq!(tag, CedraCoinType::type_tag());
        assert!(FaAddress::from_type_tag(&tag).is_native_cedra());
        assert!(!FaAddress::from_type_tag(&tag).use_fee_v2());
    }

    #[test]
    fn fa_type_tag_uses_lowercase_symbol_as_module() {
        let addr = AccountAddress::from_hex_literal(
            "0xc745ffa4f97fa9739fae0cb173996f70bb8e4b0310fa781ccca2f7dc13f7db06",
        )
        .unwrap();
        let fa = FaAddress::new(addr, b"USDCT".to_vec());
        let TypeTag::Struct(s) = fa.to_type_tag() else {
            panic!("expected struct tag");
        };
        assert_eq!(s.address, addr);
        assert_eq!(s.module.as_str(), "usdct");
        assert_eq!(s.name.as_str(), "USDCT");
        let decoded = FaAddress::from_type_tag(&TypeTag::Struct(s));
        assert_eq!(decoded.address, addr);
        assert_eq!(decoded.symbol, b"USDCT");
        assert!(decoded.use_fee_v2());
    }

    #[test]
    fn from_type_tag_ignores_legacy_module_except_cedra() {
        let addr = AccountAddress::from_hex_literal("0x42").unwrap();
        let tag = TypeTag::Struct(Box::new(StructTag {
            address: addr,
            module: Identifier::new("other_mod").unwrap(),
            name: Identifier::new("USDCT").unwrap(),
            type_args: vec![],
        }));
        let fa = FaAddress::from_type_tag(&tag);
        assert_eq!(fa.address, addr);
        assert_eq!(fa.symbol, b"USDCT");
    }

    #[test]
    fn empty_encodes_as_bool_type_tag() {
        let tag = FaAddress::empty().to_type_tag();
        assert_eq!(tag, TypeTag::Bool);
        assert!(FaAddress::from_type_tag(&tag).is_empty());
        assert!(!FaAddress::from_type_tag(&tag).use_fee_v2());
    }
}
