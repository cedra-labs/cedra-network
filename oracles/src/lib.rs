// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

// mod agg_trx_producer;
// pub mod epoch_manager;
// pub mod network;
pub mod network_interface;
// pub mod transcript_aggregation;
pub mod types;
pub mod fetch_price;
pub mod utils;
pub mod whitelist;

use url::Url;
use fetch_price::{OraclePriceList};
use whitelist::Whitelist;
use cedra_types::{
    indexer::indexer_db_reader::IndexerReader,
};

use cedra_storage_interface::{
    DbReader,
};
use std::sync::Arc;

// use crate::{network::NetworkTask, network_interface::OraclesNetworkClient};
use cedra_config::config::{ReliableBroadcastConfig, SafetyRulesConfig};
use cedra_event_notifications::{
    DbBackedOnChainConfig, EventNotificationListener, ReconfigNotificationListener,
};
use cedra_network::application::interface::{NetworkClient, NetworkServiceEvents};
use cedra_validator_transaction_pool::VTxnPoolState;
use move_core_types::account_address::AccountAddress;
use tokio::runtime::Runtime;
use cedra_types::oracles::GLOBAL_ORCLE_PRICES;

/*
    1. get whitelist coins with there metadata.
    2. fetch stablecoins price list.
*/
pub async fn start_oracle(db_reader: Arc<dyn DbReader>, indexer_reader: Option<Arc<dyn IndexerReader>>) {
    let whitelist = Whitelist::new(db_reader, indexer_reader);

    println!("{:?}", whitelist.get_whitelist());

    let oracle = OraclePriceList::new(
        Url::parse("https://dev-price-seed.cedra.dev/price-feed").unwrap(), 
        whitelist,
    );

    
    oracle.update_stablecoin_price_list().await;
    println!("{:?}", oracle.get_all());

    let mut prices = GLOBAL_ORCLE_PRICES.write().unwrap();
    *prices = oracle.get_all();

    /* 
        TODO: run in the backgroud.
        1. How to correctly call and run in the backgroud oracle.update_pricelist();
        2. How to correctly call and run in the backgroud whitelist.update_whitelist();
    */ 
}

pub fn start_oracles_runtime(
    my_addr: AccountAddress,
    safety_rules_config: &SafetyRulesConfig,
    // network_client: NetworkClient<DKGMessage>,
    // network_service_events: NetworkServiceEvents<DKGMessage>,
    reconfig_events: ReconfigNotificationListener<DbBackedOnChainConfig>,
    oracle_upddate_events: EventNotificationListener,
    vtxn_pool: VTxnPoolState,
) -> Runtime {
    // TODO: fix - EventNotificationError":"Unexpected error encountered: Channel is closed"
    let runtime = cedra_runtimes::spawn_named_runtime("oracles".into(), Some(4));
    // let oracles_network_client = OraclesNetworkClient::new(network_client);

    // let oracles_epoch_manager = EpochManager::new(
    //     safety_rules_config,
    //     my_addr,
    //     reconfig_events,
    //     oracles_update_events,
    //     oracles_network_client,
    //     vtxn_pool,
    // );
    // let (network_task, network_receiver) = NetworkTask::new(network_service_events, self_receiver);
    // runtime.spawn(network_task.start());
    // runtime.spawn(oracles_epoch_manager.start(network_receiver));
    runtime
}
