use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use cedra_types::{
    oracles::{get_global_reader, PriceInfo, PriceReader},
    validator_txn::{Topic, ValidatorTransaction},
};
use cedra_validator_transaction_pool::{TxnGuard, VTxnPoolState};

pub struct EpochManager {
    vtxn_pool: VTxnPoolState,
    published_prices: HashMap<Vec<u8>, u64>,
    _permanent_guards: Vec<TxnGuard>,
}

impl EpochManager {
    pub fn new(vtxn_pool: VTxnPoolState) -> Self {
        Self {
            vtxn_pool,
            published_prices: HashMap::new(),
            _permanent_guards: Vec::new(),
        }
    }

    pub async fn fetch_and_update_txn_pool(&mut self) -> Result<()> {
        let reader = get_global_reader();
        let all_prices = reader.get_all_prices();

        for price_info in all_prices {
            let needs_update = match self.published_prices.get(&price_info.fa_address) {
                Some(&published_price) => published_price != price_info.price,
                None => true,
            };

            if needs_update {
                // Create unique topic for each coin
                let hex_address = hex::encode(&price_info.fa_address);
                let topic = Topic::from(format!("oracle_price_{}", hex_address));
                let txn = ValidatorTransaction::PriceUpdate(price_info.clone());
                let guard = self.vtxn_pool.put(topic, Arc::new(txn), None);

                self.published_prices
                    .insert(price_info.fa_address.clone(), price_info.price);
                self._permanent_guards.push(guard);
            }
        }
        Ok(())
    }
}
