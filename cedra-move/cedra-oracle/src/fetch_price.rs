use move_core_types::{
    language_storage::{TypeTag},
};
use anyhow::Result;
use std::{
     collections::HashMap, str::FromStr, sync::RwLock
};

use cedra_rest_client::oracle::{OracleClient};
use url::Url;
use crate::{utils::get_adjusted_price_u64, whitelist::Whitelist};

// OraclePriceList contains stablecoin price list and updates coin prices.
pub struct OraclePriceList {
    price_list: RwLock<Vec<StablecoinPrice>>,
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
    pub fn get_all(&self) -> Vec<StablecoinPrice> {
        let list = self.price_list.read().unwrap();
        list.clone()
    }

    // upsert_price add new or update existing stablecoin price by fa_address.
    fn upsert_price(&self, price: StablecoinPrice) {
        let mut list = self.price_list.write().unwrap();
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
                    let fa_address = TypeTag::from_str(address).unwrap();
                    // skip coin that isn't in whitelist.
                    if !self.whitelist.exist(fa_address.clone()) {
                        continue;
                    }                    
                   
                    let decimals: u8 = 8; // TODO:...
                    let coin_price = StablecoinPrice::new(fa_address, get_adjusted_price_u64(*price, decimals), decimals);
                    self.upsert_price(coin_price);
                }
            }
            Err(e) => {
                eprintln!("Failed to update stablecoin price list: {:?}", e);
            }
        };

        println!("{:?}", self.price_list)
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

// StablecoinPrice represents stablecoin address, their price and number of decimals.
#[derive(Debug, Clone)]
pub struct StablecoinPrice {
    fa_address: TypeTag,
    price: u64,
    decimals: u8,
}

impl StablecoinPrice {
    pub fn new(fa_address: TypeTag, price: u64, decimals: u8) -> Self {
        Self {
            fa_address,
            price,
            decimals,
        }
    }
}