// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

pub mod manager;
pub mod whitelist;

use manager::OraclePriceManager;
use whitelist::Whitelist;

use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use cedra_event_notifications::EventNotificationListener;
use cedra_storage_interface::DbReader;
use cedra_types::{chain_id::ChainId, indexer::indexer_db_reader::IndexerReader};
use cedra_validator_transaction_pool::VTxnPoolState;

pub fn start_oracles_runtime(
    auth_key: Option<String>,
    oracles_updated_events: Option<EventNotificationListener>,
    vtxn_pool: VTxnPoolState,
    db_reader: Arc<dyn DbReader>,
    indexer_reader: Option<Arc<dyn IndexerReader>>,
    chain_id: ChainId,
) -> Runtime {
    let runtime = cedra_runtimes::spawn_named_runtime("oracles".into(), Some(4));
    if auth_key.is_some() && oracles_updated_events.is_some() {
    let whitelist = Arc::new(Whitelist::new(db_reader, indexer_reader));

    let mut oracle_price_manager = OraclePriceManager::new(
        auth_key.unwrap(),
        Arc::clone(&whitelist),
        vtxn_pool,
        oracles_updated_events.unwrap(),
        chain_id,
    );

    let (price_tx, price_rx) = mpsc::channel(1000);

    runtime.spawn(async move {
        if let Err(e) = oracle_price_manager.run(price_rx, price_tx).await {
            eprintln!("Oracle price manager start failed: {:?}", e);
        }
    });
    } else {
         tracing::warn!("Oracle runtime not started: auth key missing or events not exist on move");
    }

    runtime
}
