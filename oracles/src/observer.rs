use crate::whitelist::Whitelist;
use cedra_validator_transaction_pool::{VTxnPoolState,TxnGuard};
use rand::Rng;

use anyhow::Result;
use std::{sync::Arc,collections::HashMap};
use url::Url;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use cedra_rest_client::oracle::OracleClient;
use cedra_types::{
    validator_txn::{ValidatorTransaction,Topic},
    oracles::{update_global_price, PriceInfo, PriceUpdated},CedraCoinType, CoinType
};
use cedra_channels::cedra_channel;
use tracing::info;

use cedra_channels::message_queues::QueueStyle;
use cedra_event_notifications::{EventNotification, EventNotificationListener};
use futures::StreamExt;


// OracleObserver fetch stablecoin price list and updates InMemoryPriceStorage.
pub struct OracleObserver {
    whitelist: Whitelist,
    vtxn_pool: VTxnPoolState,
     oracles_updated_events: EventNotificationListener,
    _permanent_guards: Mutex<Vec<TxnGuard>>,}

impl OracleObserver {
    pub fn new(whitelist: Whitelist, vtxn_pool: VTxnPoolState, oracles_updated_events:EventNotificationListener ) -> Self {
        Self {
            whitelist,
            vtxn_pool,
            oracles_updated_events,
            _permanent_guards: Mutex::new(Vec::new()),        }
    }

    pub async fn run_price_sync_loop(&self) {
        loop {
            if let Err(e) = self.sync_prices_to_validator_pool().await {
                eprintln!("[OracleObserver] Failed to sync prices: {:?}", e);
            }

            sleep(Duration::from_secs(10)).await;
        }
    }


  pub async fn run_random_price_generator(&mut self) {


    loop {
            tokio::select! {
                // Handle events
event = self.oracles_updated_events.select_next_some()     => {

                    if let Err(e) = self.process_onchain_event(event).await {
                        eprintln!("[OracleObserver] Error processing on-chain event: {:?}", e);
                    }
                }
                // Generate random prices periodically
                _ = sleep(Duration::from_secs(10)) => {
                    if let Err(e) = self.generate_and_update_random_prices().await {
                        eprintln!("[OracleObserver] Error generating random prices: {:?}", e);
                    }
                }
            }
        }             }

    async fn generate_and_update_random_prices(&self) -> Result<()> {
  let whitelist = self.whitelist.get_whitelist();
        let mut price_feed: Vec<PriceInfo> = Vec::new();

        for coin_info in whitelist {
            // Skip coins that aren't in whitelist (fixed logic)
            if !self.whitelist.exist(coin_info.fa_address.clone()) 
                && coin_info.fa_address != CedraCoinType::type_tag().to_string() 
            {
                println!("[RandomPriceGenerator] Skipping non-whitelisted coin: {}", coin_info.fa_address);
             return Ok(());
            }

            let random_price = rand::thread_rng().gen_range(10.0,20.0);        
            price_feed.push(PriceInfo {
                fa_address: coin_info.get_fa_address().into_bytes(),
                price: (random_price * 1_000.0) as u64,
                decimals: 6,
            });
        }

        if !price_feed.is_empty() {
            println!("[RandomPriceGenerator] Generated prices for {} coins", price_feed.len());
            self.update_validator_txn_pool(price_feed).await?;
        } else {
            println!("[RandomPriceGenerator] No valid coins to generate prices for");
        }

        Ok(())
        }    
    // Main method that combines price fetching and validator transaction pool updates
    async fn sync_prices_to_validator_pool(&self) -> Result<()> {
        // Step 1: Fetch latest prices from oracle
        // let price_feed = self.get_stablecoins_list().await?;
              // Step 2: Sync updated prices to validator transaction pool
        // self.update_validator_txn_pool().await?;
        
        Ok(())
    }

    /// Process on-chain price_storage updated events
    async fn process_onchain_event(&mut self, notification: EventNotification) -> Result<()> {
        let EventNotification { subscribed_events, .. } = notification;
        
        for event in subscribed_events {
            println!("[OracleObserver] Processing event: {:?}", event);
            
            if let Ok(price_storage_event) = PriceUpdated::try_from(&event) {
                    println!("[OracleObserver] Successfully parsed PriceUpdated event: {:?}", price_storage_event.prices);
                        update_global_price(price_storage_event);
                    } else {
                    eprintln!("[OracleObserver] Failed to parse PriceUpdated event");
                    // Log the raw event data for debugging
                    println!("[OracleObserver] Raw event data: {:?}", event);
                }
            }
        Ok(())
    }


    async fn update_validator_txn_pool(&self, prices: Vec<PriceInfo>) -> Result<()> {
    let mut permanent_guards = self._permanent_guards.lock().await;

    let txn = ValidatorTransaction::PriceUpdate(prices);
    let topic = Topic::from("oracle_prices");
    let guard = self.vtxn_pool.put(topic, Arc::new(txn), None);
    
    permanent_guards.push(guard);
            Ok(())
    }
}
