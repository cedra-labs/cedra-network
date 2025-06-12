// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{driver_factory::DriverFactory, metadata_storage::PersistentMetadataStorage};
use cedra_config::{
    config::{
        RocksdbConfigs, StorageDirPaths, BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD, NO_OP_STORAGE_PRUNER_CONFIG,
    },
    utils::get_genesis_txn,
};
use cedra_consensus_notifications::new_consensus_notifier_listener_pair;
use cedra_data_client::client::CedraDataClient;
use cedra_data_streaming_service::streaming_client::new_streaming_service_client_listener_pair;
use cedra_db::CedraDB;
use cedra_event_notifications::EventSubscriptionService;
use cedra_executor::chunk_executor::ChunkExecutor;
use cedra_executor_test_helpers::bootstrap_genesis;
use cedra_genesis::test_utils::test_config;
use cedra_infallible::RwLock;
use cedra_mempool_notifications::new_mempool_notifier_listener_pair;
use cedra_network::application::{interface::NetworkClient, storage::PeersAndMetadata};
use cedra_storage_interface::DbReaderWriter;
use cedra_storage_service_client::StorageServiceClient;
use cedra_temppath::TempPath;
use cedra_time_service::TimeService;
use cedra_vm::cedra_vm::CedraVMBlockExecutor;
use futures::{FutureExt, StreamExt};
use std::{collections::HashMap, sync::Arc};

#[test]
fn test_new_initialized_configs() {
    // Create a test database
    let tmp_dir = TempPath::new();
    let db = CedraDB::open(
        StorageDirPaths::from_path(&tmp_dir),
        false,
        NO_OP_STORAGE_PRUNER_CONFIG,
        RocksdbConfigs::default(),
        false, /* indexer */
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
        None,
    )
    .unwrap();
    let (_, db_rw) = DbReaderWriter::wrap(db);

    // Bootstrap the database
    let (node_config, _) = test_config();
    bootstrap_genesis::<CedraVMBlockExecutor>(&db_rw, get_genesis_txn(&node_config).unwrap())
        .unwrap();

    // Create mempool and consensus notifiers
    let (mempool_notifier, _) = new_mempool_notifier_listener_pair(100);
    let (_, consensus_listener) = new_consensus_notifier_listener_pair(0);

    // Create the event subscription service and a reconfig subscriber
    let mut event_subscription_service =
        EventSubscriptionService::new(Arc::new(RwLock::new(db_rw.clone())));
    let mut reconfiguration_subscriber = event_subscription_service
        .subscribe_to_reconfigurations()
        .unwrap();

    // Create the storage service notifier and listener
    let (storage_service_notifier, _storage_service_listener) =
        cedra_storage_service_notifications::new_storage_service_notifier_listener_pair();

    // Create a test streaming service client
    let (streaming_service_client, _) = new_streaming_service_client_listener_pair();

    // Create a test cedra data client
    let network_client = StorageServiceClient::new(NetworkClient::new(
        vec![],
        vec![],
        HashMap::new(),
        PeersAndMetadata::new(&[]),
    ));
    let (cedra_data_client, _) = CedraDataClient::new(
        node_config.state_sync.cedra_data_client,
        node_config.base.clone(),
        TimeService::mock(),
        db_rw.reader.clone(),
        network_client,
        None,
    );

    // Create the state sync driver factory
    let chunk_executor = Arc::new(ChunkExecutor::<CedraVMBlockExecutor>::new(db_rw.clone()));
    let metadata_storage = PersistentMetadataStorage::new(tmp_dir.path());
    let _ = DriverFactory::create_and_spawn_driver(
        true,
        &node_config,
        node_config.base.waypoint.waypoint(),
        db_rw,
        chunk_executor,
        mempool_notifier,
        storage_service_notifier,
        metadata_storage,
        consensus_listener,
        event_subscription_service,
        cedra_data_client,
        streaming_service_client,
        TimeService::mock(),
    );

    // Verify the initial configs were notified
    assert!(reconfiguration_subscriber
        .select_next_some()
        .now_or_never()
        .is_some());
}
