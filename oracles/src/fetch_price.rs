use move_core_types::{
    language_storage::{TypeTag},
};
use anyhow::Result;
use std::{
     collections::HashMap, str::FromStr, sync::RwLock
};

use cedra_types::{
    CedraCoinType, CoinType
};

// PRICE_DECIMALS represents decimals value that helps convert price value from f64 to u64.
const PRICE_DECIMALS: u8 = 8;

use cedra_rest_client::oracle::{OracleClient};
use url::Url;
use crate::{utils::get_adjusted_price_u64, whitelist::{Whitelist, FAMetadata}};

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

        // update existing price or add a new value.
        if let Some(existing) = list.iter_mut().find(|p| p.stablecoin.get_fa_address() == price.stablecoin.get_fa_address()) {
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
                    if !self.whitelist.exist(fa_address.clone()) && fa_address != CedraCoinType::type_tag() {
                        continue;
                    }                    
                   
                    let metatdata = self.whitelist.get_fa_address_metadata(fa_address);
                    
                    let coin_price = StablecoinPrice::new(metatdata, get_adjusted_price_u64(*price, PRICE_DECIMALS));
                    self.upsert_price(coin_price);
                }
            }
            Err(e) => {
                eprintln!("Failed to update stablecoin price list: {:?}", e);
            }
        };

        println!("-----------------------------------");
        println!("-----------------------------------");
        println!("{:?}", self.price_list);
        println!("-----------------------------------");
        println!("-----------------------------------");
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
    stablecoin: FAMetadata,
    price: u64,
    price_decimals: u8,
}

impl StablecoinPrice {
    pub fn new(stablecoin: FAMetadata, price: u64) -> Self {
        Self {
            stablecoin,
            price,
            price_decimals: PRICE_DECIMALS,
        }
    }

    pub fn get_price(&self) -> u64 {
        self.price.clone()
    }

    pub fn get_price_decimals(&self) -> u8 {
        self.price_decimals.clone()
    }
}