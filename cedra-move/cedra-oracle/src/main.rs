//entry point
use anyhow::Result;
use url::Url;
use fetch_price::OraclePriceList;
use whitelist::Whitelist;

// mods...
pub mod fetch_price;
pub mod utils;
pub mod whitelist;

/*
    1. get whitelist coins.
    2. fetch stablecoins metadata.
    3. fetch stablecoins price list.
*/

#[tokio::main] 
async  fn main() {
    let whitelist = Whitelist::new();

    let oracle = OraclePriceList::new(
        Url::parse("https://dev-price-seed.cedra.dev/price-feed").unwrap(), 
        whitelist,
    );

    oracle.update_stablecoin_price_list().await;
    // println!("{:?}", prices)
}