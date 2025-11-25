// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

pub mod observer;
pub mod whitelist;
pub mod network;
pub mod network_interface;
pub mod counters;
pub mod types;

use std::sync::Arc;
use tokio::{runtime::Runtime};
use url::Url;

use observer::OracleObserver;
use whitelist::Whitelist;
use network_interface::OracleNetworkClient;
use network::NetworkTask;

use cedra_storage_interface::DbReader;
use cedra_types::{indexer::indexer_db_reader::IndexerReader, oracles::GLOBAL_PRICE_STORAGE};
use cedra_validator_transaction_pool::VTxnPoolState;
use cedra_event_notifications::{
     EventNotificationListener,
};
use cedra_network::application::interface::{NetworkClient, NetworkServiceEvents};
pub use types::OracleMessage;

pub fn start_oracles_runtime(
    network_client: NetworkClient<OracleMessage>,
    network_service_events: NetworkServiceEvents<OracleMessage>,
    oracles_updated_events: EventNotificationListener,
    vtxn_pool: VTxnPoolState,
    db_reader: Arc<dyn DbReader>,
    indexer_reader: Option<Arc<dyn IndexerReader>>,
) -> Runtime {
    let _ = &GLOBAL_PRICE_STORAGE;
    let whitelist = Whitelist::new(db_reader, indexer_reader);

    let mut observer = OracleObserver::new(
        whitelist,
        vtxn_pool,
        oracles_updated_events
    );

    let runtime = cedra_runtimes::spawn_named_runtime("oracles".into(), Some(4));
    let (self_sender, self_receiver) = cedra_channels::new(1_024, &counters::PENDING_SELF_MESSAGES);
    let oracle_network_client = OracleNetworkClient::new(network_client);
    let (network_task, network_receiver) = NetworkTask::new(network_service_events, self_receiver);

    runtime.spawn(network_task.start());
 
    // Spawn the oracle in the background
    runtime.spawn(async move {
        // observer_clone.run_price_sync_loop().await;
        observer.run_random_price_generator().await;

    });
    runtime
}
