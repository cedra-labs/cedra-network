use anyhow::Result;
use std::{
     collections::HashMap, sync::RwLock
};
use tokio::time::{sleep, Duration};
use cedra_types::{
    CedraCoinType, CoinType, oracles::{PriceInfo, DEFAULT_DECIMALS},
};

use cedra_rest_client::oracle::{OracleClient};
use url::Url;
use crate::{utils::get_adjusted_price_u64, whitelist::{Whitelist}};

// OraclePriceList contains stablecoin price list and updates coin prices.
pub struct OraclePriceList {
    price_list: RwLock<Vec<PriceInfo>>,
    oracle_client: OracleClient,
    whitelist: Whitelist,
}

impl OraclePriceList {
    pub fn new(oracle_url: Url, whitelist: Whitelist) -> Self {
        Self {
            price_list: RwLock::new(Vec::new()),
            oracle_client: OracleClient::new(oracle_url.clone()),
            whitelist: whitelist
        }
    }

    // get_all returns a list of stable coins with their prices.
    pub fn get_all(&self) -> Vec<PriceInfo> {
        let list = self.price_list.read().unwrap();
        list.clone()
    }

    // upsert_price add new or update existing stablecoin price by fa_address.
    fn upsert_price(&self, price: PriceInfo) {
        let mut list = self.price_list.write().unwrap();

        // update existing price or add a new value.
        if let Some(existing) = list.iter_mut().find(|p| p.fa_address == price.fa_address) {
            *existing = price;
        } else {
            list.push(price);
        }
    }

    // update_stablecoin_price_list updates stablecoin prices.
    pub async fn update_stablecoin_price_list(&self) {
        match self.get_stablecoin_price_list().await {
            Ok(price_feed) => {
                for (address, price) in &price_feed {
                    // skip coin that isn't in whitelist.
                    if !self.whitelist.exist(address.clone()) && address.clone() != CedraCoinType::type_tag().to_string() {
                        continue;
                    }                    
                   
                    let metadata = self.whitelist.get_fa_address_coin_info(address.clone());
                    
                    let coin_price = PriceInfo::new(
                        metadata.get_fa_address(), 
                        get_adjusted_price_u64(*price, DEFAULT_DECIMALS), 
                        metadata.get_decimals());
                    self.upsert_price(coin_price);
                }
            }
            Err(e) => {
                eprintln!("Failed to update stablecoin price list: {:?}", e);
            }
        };
    }

    // update_update_pricelist - update pricelist data (should be run as a background task).
    pub async fn update_pricelist(&self) {
        loop {
            self.update_stablecoin_price_list().await;
            sleep(Duration::from_secs(10)).await;
        }
    }

    // get_stablecoin_price_list calls oracle client and returns stablecons price list as a HashMap result.
    async fn get_stablecoin_price_list(&self) -> Result<HashMap<String, f64>> {
         match self.oracle_client.get_price_list().await {
            Ok(response) => {
                let price_feed: HashMap<String, f64> =  response.json().await.unwrap();
                Ok(price_feed)
            },
            Err(e) => {
                eprintln!("Failed to fetch price list: {:?}", e);
                Ok(HashMap::new())
            },
        }
    }
}

// StablecoinPrice represents FA stablecoin metadata, their price and number of decimals for price scaling.
// #[derive(Debug, Clone)]
// pub struct StablecoinPrice {
//     stablecoin: PriceInfo,
//     price: u64,
//     decimals: u8,
// }

// impl StablecoinPrice {
//     pub fn new(stablecoin: PriceInf, price: u64) -> Self {
//         Self {
//             stablecoin,
//             price,
//             decimals: DEFAULT_DECIMALS,
//         }
//     }

//     pub fn get_price(&self) -> u64 {
//         self.price.clone()
//     }

//     pub fn get_decimals(&self) -> u8 {
//         self.decimals.clone()
//     }
// }