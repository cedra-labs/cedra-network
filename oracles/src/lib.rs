// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

pub mod epoch_manager;
pub mod observer;
pub mod whitelist;

use std::sync::Arc;
use tokio::{runtime::Runtime, time::sleep};
use url::Url;

use epoch_manager::EpochManager;
use observer::OracleObserver;
use whitelist::Whitelist;

use cedra_storage_interface::DbReader;
use cedra_types::{indexer::indexer_db_reader::IndexerReader, oracles::GLOBAL_PRICE_STORAGE};
use cedra_validator_transaction_pool::VTxnPoolState;

pub fn start_oracles_runtime(
    vtxn_pool: VTxnPoolState,
    db_reader: Arc<dyn DbReader>,
    indexer_reader: Option<Arc<dyn IndexerReader>>,
) -> Runtime {
    let _ = &GLOBAL_PRICE_STORAGE;
    let whitelist = Whitelist::new(db_reader, indexer_reader);

    let observer = Arc::new(OracleObserver::new(
        Url::parse("https://dev-price-seed.cedra.dev/price-feed").unwrap(),
        whitelist,
    ));

    let runtime = cedra_runtimes::spawn_named_runtime("oracles".into(), Some(4));

    // Spawn the oracle loop in the background
    runtime.spawn(async move {
        let mut epoch_manager = EpochManager::new(vtxn_pool.clone());

        loop {
            observer.update_price_storage().await;

            if let Err(e) = epoch_manager.fetch_and_update_txn_pool().await {
                eprintln!("[OraclesRuntime] Failed to update txn pool: {:?}", e);
            }

            //todo: make sure sleep is working on this runtime
            sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });
    runtime
}
