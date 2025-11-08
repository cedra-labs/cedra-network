// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_crypto_derive::{BCSCryptoHash, CryptoHasher};
use move_core_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveStructType,
};
use poem_openapi_derive::Object;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    sync::{Arc, LazyLock, RwLock},
};

pub const DEFAULT_DECIMALS: u8 = 8;

/// Rust reflection of `0x1::price_storage::PriceInfo`
#[derive(
    Clone, Hash, Serialize, Object, Deserialize, PartialEq, Eq, CryptoHasher, BCSCryptoHash,
)]
pub struct PriceInfo {
    /// Address of the fungible asset (FA) as UTF-8 string bytes
    #[serde(with = "serde_bytes")]
    pub fa_address: Vec<u8>,
    /// Scaled price value (price * 10^decimals)
    pub price: u64,
    /// Number of decimals used for scaling
    pub decimals: u8,
}

impl PriceInfo {
    pub fn new(fa_address: impl Into<Vec<u8>>, price: u64) -> Self {
        Self {
            fa_address: fa_address.into(),
            price,
            decimals: DEFAULT_DECIMALS,
        }
    }

    pub fn with_decimals(fa_address: impl Into<Vec<u8>>, price: u64, decimals: u8) -> Self {
        Self {
            fa_address: fa_address.into(),
            price,
            decimals,
        }
    }

    pub fn fa_address_hex(&self) -> String {
        hex::encode(&self.fa_address)
    }

    pub fn actual_price(&self) -> f64 {
        self.price as f64 / 10f64.powi(self.decimals as i32)
    }
}

impl Debug for PriceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PriceInfo")
            .field("fa_address", &self.fa_address_hex())
            .field("price", &self.price)
            .field("decimals", &self.decimals)
            .field("actual_price", &self.actual_price())
            .finish()
    }
}

impl MoveStructType for PriceInfo {
    const MODULE_NAME: &'static IdentStr = ident_str!("price_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PriceInfo");
}

/// Read-only interface for price queries
pub trait PriceReader: Send + Sync {
    fn get_price(&self, fa_address: &[u8]) -> Option<PriceInfo>;
    fn get_price_by_type(&self, stablecoin: TypeTag) -> Option<PriceInfo>;
    fn get_all_prices(&self) -> Vec<PriceInfo>;
}

/// Write-only interface for price updates
pub trait PriceWriter: Send + Sync {
    fn update_price(&self, price: PriceInfo);
    fn update_prices(&self, prices: Vec<PriceInfo>);
    fn remove_price(&self, fa_address: &[u8]) -> bool;
}

/// Rust implememntation of `0x1::price_storage::PriceStorage` target logic
/// Combined storage that implements both Reader and Writer interfaces
pub struct InMemoryPriceStorage {
    prices: Arc<RwLock<HashMap<Vec<u8>, PriceInfo>>>,
}

impl InMemoryPriceStorage {
    pub fn new() -> Self {
        Self {
            prices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_default_prices() -> Self {
        let default_prices = vec![
            PriceInfo::new("0x1::cedra_coin::CedraCoin".to_string(), 100000000),
            PriceInfo::new(
                "0xc745ffa4f97fa9739fae0cb173996f70bb8e4b0310fa781ccca2f7dc13f7db06::usdct::USDCT"
                    .to_string(),
                100000000,
            ),
            PriceInfo::new(
                "0xca0746f983f7d03891c5e2dab8e321784357e687848b3840ae4cf5cc619dba7a::usdt::USDT"
                    .to_string(),
                100000000,
            ),
            PriceInfo::new("0x1::example_coin::ExampleCoin".to_string(), 150000000),
        ];

        Self::with_initial_prices(default_prices)
    }

    pub fn with_initial_prices(prices: Vec<PriceInfo>) -> Self {
        let mut map = HashMap::new();
        for price in prices {
            map.insert(price.fa_address.clone(), price);
        }
        Self {
            prices: Arc::new(RwLock::new(map)),
        }
    }

    /// Create a read-only view of this storage
    pub fn reader(&self) -> impl PriceReader + '_ {
        PriceStorageReader { storage: self }
    }

    /// Create a write-only view of this storage  
    pub fn writer(&self) -> impl PriceWriter + '_ {
        PriceStorageWriter { storage: self }
    }

    /// Get both reader and writer interfaces
    pub fn interfaces(&self) -> (impl PriceReader + '_, impl PriceWriter + '_) {
        (self.reader(), self.writer())
    }
}

struct PriceStorageReader<'a> {
    storage: &'a InMemoryPriceStorage,
}

impl<'a> PriceReader for PriceStorageReader<'a> {
    fn get_price(&self, fa_address: &[u8]) -> Option<PriceInfo> {
        self.storage
            .prices
            .read()
            .ok()
            .and_then(|prices| prices.get(fa_address).cloned())
    }

    fn get_price_by_type(&self, stablecoin: TypeTag) -> Option<PriceInfo> {
        let stablecoin_bytes = bcs::to_bytes(&stablecoin).ok()?;
        self.get_price(&stablecoin_bytes)
    }

    fn get_all_prices(&self) -> Vec<PriceInfo> {
        self.storage
            .prices
            .read()
            .map(|prices| prices.values().cloned().collect())
            .unwrap_or_default()
    }
}

struct PriceStorageWriter<'a> {
    storage: &'a InMemoryPriceStorage,
}

impl<'a> PriceWriter for PriceStorageWriter<'a> {
    fn update_price(&self, price: PriceInfo) {
        if let Ok(mut prices) = self.storage.prices.write() {
            prices.insert(price.fa_address.clone(), price);
        }
    }

    fn update_prices(&self, prices: Vec<PriceInfo>) {
        if let Ok(mut price_map) = self.storage.prices.write() {
            for price in prices {
                price_map.insert(price.fa_address.clone(), price);
            }
        }
    }

    fn remove_price(&self, fa_address: &[u8]) -> bool {
        if let Ok(mut prices) = self.storage.prices.write() {
            prices.remove(fa_address).is_some()
        } else {
            false
        }
    }
}

//todo: change to ::new on devnet after ocalnet tests
pub static GLOBAL_PRICE_STORAGE: LazyLock<Arc<InMemoryPriceStorage>> =
    LazyLock::new(|| Arc::new(InMemoryPriceStorage::with_default_prices()));

pub fn get_global_reader() -> impl PriceReader + 'static {
    GLOBAL_PRICE_STORAGE.reader()
}

pub fn get_global_writer() -> impl PriceWriter + 'static {
    GLOBAL_PRICE_STORAGE.writer()
}

pub fn get_price_info(stablecoin: TypeTag) -> Option<PriceInfo> {
    get_global_reader().get_price_by_type(stablecoin)
}

pub fn update_global_price(price: PriceInfo) {
    get_global_writer().update_price(price);
}

pub static PRICE_INFO_TYPE_TAG: LazyLock<TypeTag> =
    LazyLock::new(|| TypeTag::Struct(Box::new(PriceInfo::struct_tag())));
