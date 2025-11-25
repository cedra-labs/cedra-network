// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0


use crate::{move_utils::as_move_value::AsMoveValue};
use cedra_crypto_derive::{BCSCryptoHash, CryptoHasher};
use move_core_types::{value::{MoveStruct, MoveValue},
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveStructType,
};
use poem_openapi_derive::Object;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    sync::{RwLock, LazyLock, atomic::{AtomicU64, Ordering}},
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
    fn get_price(&self, fa_address: String) -> Option<PriceInfo>;
    fn get_price_by_type(&self, stablecoin: TypeTag) -> Option<PriceInfo>;
    fn get_all_prices(&self) -> Vec<PriceInfo>;
    fn get_version(&self) -> u64;
}

/// Write-only interface for price updates
pub trait PriceWriter: Send + Sync {
    fn update_prices(&mut self, update_event_data: PriceUpdated);
}

/// Rust implememntation of `0x1::price_storage::PriceStorage` target logic
/// Combined storage that implements both Reader and Writer interfaces
#[derive(Serialize, Deserialize)]
pub struct InMemoryPriceStorage {
    pub prices: RwLock<Vec<PriceInfo>>,
    pub version: AtomicU64,
}

impl InMemoryPriceStorage {
    pub fn new() -> Self {
        Self {
  prices: RwLock::new(Vec::new()),
  version: AtomicU64::new(0),        }
    }

    /// Create a read-only view of this storage
    pub fn reader(&self) -> impl PriceReader + '_ {
        PriceStorageReader { storage: self }
    }

    /// Create a write-only view of this storage  
    pub fn writer(&self) -> impl PriceWriter + '_ {
        PriceStorageWriter { storage: self }
    }

}

struct PriceStorageReader<'a> {
    storage: &'a InMemoryPriceStorage,
}

impl<'a> PriceReader for PriceStorageReader<'a> {
    fn get_price(&self, fa_address: String) -> Option<PriceInfo> {

            let reader = get_global_reader();
    
    let prices = reader.get_all_prices();
let stablecoin_str = fa_address.to_string();

        println!("ROBERTO3: {:?}", prices);
        println!("ROBERTO4: {:?}", fa_address);

        prices
            .iter()
            .find(|p| {
        let fa_str = String::from_utf8(p.fa_address.clone()).unwrap_or_default();
                println!("ROBERTO5: {:?}", fa_str);

             fa_str == fa_address})
            .cloned()
    }

    fn get_price_by_type(&self, stablecoin: TypeTag) -> Option<PriceInfo> {
let stablecoin_str = stablecoin.to_string();
        println!("ROBERTO: {}", stablecoin);
        println!("ROBERTO2: {:?}", stablecoin_str);
        self.get_price(stablecoin_str)
    }

    fn get_all_prices(&self) -> Vec<PriceInfo> {
        let prices = self.storage.prices.read().unwrap();
        prices.clone()
    }

      fn get_version(&self) -> u64 {

        self.storage.version.load(Ordering::SeqCst)    }
}

struct PriceStorageWriter<'a> {
    storage: &'a InMemoryPriceStorage,
}

impl<'a> PriceWriter for PriceStorageWriter<'a> {
    fn update_prices(&mut self, update_event_data: PriceUpdated) {
let PriceUpdated { prices, version } = update_event_data;
    
    {
        let mut prices_guard = self.storage.prices.write().unwrap();
        *prices_guard = prices.clone(); // Clone if needed, or move if you don't need prices after
        self.storage.version.store(version, Ordering::SeqCst);
    } // Write lock released here
    
}
}

pub static GLOBAL_PRICE_STORAGE: LazyLock<InMemoryPriceStorage> =
 LazyLock::new(|| InMemoryPriceStorage::new());

pub fn get_global_reader() -> impl PriceReader + 'static {
    GLOBAL_PRICE_STORAGE.reader()
}

pub fn get_global_writer() -> impl PriceWriter + 'static {
    GLOBAL_PRICE_STORAGE.writer()
}

pub fn get_price_info(stablecoin: TypeTag) -> Option<PriceInfo> {
    get_global_reader().get_price_by_type(stablecoin)
}

pub fn get_version() -> u64 {
    get_global_reader().get_version()
}

pub fn update_global_price(update_event_data: PriceUpdated) {
    get_global_writer().update_prices(update_event_data);
}

pub static PRICE_INFO_TYPE_TAG: LazyLock<TypeTag> =
    LazyLock::new(|| TypeTag::Struct(Box::new(PriceInfo::struct_tag())));


// Move event type `0x1::price_storage::PriceUpdated` in rust.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub struct PriceUpdated {
    pub version: u64,
    pub prices: Vec<PriceInfo>,
}

impl MoveStructType for PriceUpdated {
    const MODULE_NAME: &'static IdentStr = ident_str!("price_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PriceUpdated");
}


pub static PRICE_UPDATED_MOVE_TYPE_TAG: LazyLock<TypeTag> =
    LazyLock::new(|| TypeTag::Struct(Box::new(PriceUpdated::struct_tag())));

impl AsMoveValue for PriceInfo {
    fn as_move_value(&self) -> MoveValue {
        MoveValue::Struct(MoveStruct::Runtime(vec![
            self.fa_address.as_move_value(),
            self.price.as_move_value(),
            self.decimals.as_move_value(),        ]))
    }
}
