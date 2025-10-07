//entry point
// use anyhow::Result;
use url::Url;
use fetch_price::OraclePriceList;
use whitelist::Whitelist;
use cedra_types::{
    indexer::indexer_db_reader::IndexerReader,
};

use cedra_storage_interface::{
    DbReader,
};
use std::sync::Arc;

// mods...
pub mod fetch_price;
pub mod utils;
pub mod whitelist;

/*
    1. get whitelist coins. +
    2. fetch stablecoins metadata. - 
    3. fetch stablecoins price list. +
*/

pub async fn start_oracle(db_reader: Arc<dyn DbReader>, indexer_reader: Option<Arc<dyn IndexerReader>>) {
    let whitelist = Whitelist::new(db_reader, indexer_reader);

    let oracle = OraclePriceList::new(
        Url::parse("https://dev-price-seed.cedra.dev/price-feed").unwrap(), 
        whitelist,
    );

    oracle.update_stablecoin_price_list().await;
    // println!("{:?}", prices)
}