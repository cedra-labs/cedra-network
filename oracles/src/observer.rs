use crate::whitelist::Whitelist;

use anyhow::Result;
use std::collections::HashMap;
use url::Url;

use cedra_rest_client::oracle::OracleClient;
use cedra_types::{
    oracles::{get_global_writer, PriceInfo, PriceWriter},
    CedraCoinType, CoinType,
};

// OracleObserver fetch stablecoin price list and updates InMemoryPriceStorage.
pub struct OracleObserver {
    oracle_client: OracleClient,
    whitelist: Whitelist,
}

impl OracleObserver {
    pub fn new(oracle_url: Url, whitelist: Whitelist) -> Self {
        Self {
            oracle_client: OracleClient::new(oracle_url.clone()),
            whitelist,
        }
    }

    pub async fn update_price_storage(&self) {
        match self.get_stablecoins_list().await {
            Ok(price_feed) => {
                let writer = get_global_writer();
                let mut prices_to_update = Vec::new();

                for (address, price) in &price_feed {
                    // skip coin that isn't in whitelist.
                    if !self.whitelist.exist(address.clone())
                        && address.clone() != CedraCoinType::type_tag().to_string()
                    {
                        continue;
                    }

                    let metadata = self.whitelist.get_coin_info(address.clone());

                    let coin_price = PriceInfo::with_decimals(
                        metadata.get_fa_address(),
                        (*price as u8).into(),
                        metadata.get_decimals(),
                    );
                    prices_to_update.push(coin_price);
                }

                if !prices_to_update.is_empty() {
                    writer.update_prices(prices_to_update);
                    println!(
                        "[OBSERVER] Updated {} prices in global storage",
                        price_feed.len()
                    );
                }
            },
            Err(e) => {
                eprintln!("Failed to update stablecoin price list: {:?}", e);
            },
        };
    }

    // calls oracle client and returns stablecons price list as a HashMap result.
    async fn get_stablecoins_list(&self) -> Result<HashMap<String, f64>> {
        match self.oracle_client.get_price_list().await {
            Ok(response) => {
                let price_feed: HashMap<String, f64> = response.json().await.unwrap();
                Ok(price_feed)
            },
            Err(e) => {
                eprintln!("Failed to fetch price list: {:?}", e);
                Ok(HashMap::new())
            },
        }
    }
}
