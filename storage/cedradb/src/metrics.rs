// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_metrics_core::{
    exponential_buckets, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, HistogramVec, IntCounter, IntCounterVec, IntGauge,
    IntGaugeVec,
};
use once_cell::sync::Lazy;

pub static LEDGER_COUNTER: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "cedra_storage_ledger",
        // metric description
        "Cedra storage ledger counters",
        // metric labels (dimensions)
        &["type"]
    )
    .unwrap()
});

pub static COMMITTED_TXNS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "cedra_storage_committed_txns",
        "Cedra storage committed transactions"
    )
    .unwrap()
});

pub static LATEST_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_storage_latest_transaction_version",
        "Cedra storage latest transaction version"
    )
    .unwrap()
});

pub static LEDGER_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_storage_ledger_version",
        "Version in the latest saved ledger info."
    )
    .unwrap()
});

pub static NEXT_BLOCK_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_storage_next_block_epoch",
        "ledger_info.next_block_epoch() for the latest saved ledger info."
    )
    .unwrap()
});

pub static STATE_ITEMS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("cedra_storage_state_items", "Total number of state items.").unwrap()
});

pub static TOTAL_STATE_BYTES: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_storage_total_state_bytes",
        "Total size in bytes of all state items."
    )
    .unwrap()
});

pub static PRUNER_WINDOW: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "cedra_storage_prune_window",
        // metric description
        "Cedra storage prune window",
        // metric labels (dimensions)
        &["pruner_name",]
    )
    .unwrap()
});

/// DB pruner least readable versions
pub static PRUNER_VERSIONS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "cedra_pruner_versions",
        // metric description
        "Cedra pruner versions",
        // metric labels (dimensions)
        &["pruner_name", "tag"]
    )
    .unwrap()
});

/// Pruner batch size. For ledger pruner, this means the number of versions to be pruned at a time.
/// For state store pruner, this means the number of stale nodes to be pruned at a time.
pub static PRUNER_BATCH_SIZE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "pruner_batch_size",
        // metric description
        "Cedra pruner batch size",
        // metric labels (dimensions)
        &["pruner_name",]
    )
    .unwrap()
});

pub static API_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_storage_api_latency_seconds",
        // metric description
        "Cedra storage api latency in seconds",
        // metric labels (dimensions)
        &["api_name", "result"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static OTHER_TIMERS_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_storage_other_timers_seconds",
        // metric description
        "Various timers below public API level.",
        // metric labels (dimensions)
        &["name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static NODE_CACHE_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_storage_node_cache_seconds",
        // metric description
        "Latency of node cache.",
        // metric labels (dimensions)
        &["tag", "name"],
        exponential_buckets(/*start=*/ 1e-9, /*factor=*/ 2.0, /*count=*/ 30).unwrap(),
    )
    .unwrap()
});

/// Rocksdb metrics
pub static ROCKSDB_PROPERTIES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "cedra_rocksdb_properties",
        // metric description
        "rocksdb integer properties",
        // metric labels (dimensions)
        &["cf_name", "property_name",]
    )
    .unwrap()
});

/// Rocksdb metrics
pub static ROCKSDB_SHARD_PROPERTIES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "cedra_rocksdb_shard_properties",
        // metric description
        "sharded rocksdb integer properties",
        // metric labels (dimensions)
        &["shard_id", "cf_name", "property_name",]
    )
    .unwrap()
});

// Async committer gauges:
pub(crate) static LATEST_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_storage_latest_state_snapshot_version",
        "The version of the most recent snapshot."
    )
    .unwrap()
});

pub(crate) static LATEST_CHECKPOINT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_storage_latest_state_checkpoint_version",
        "The version of the most recent committed checkpoint."
    )
    .unwrap()
});

// Backup progress gauges:

pub(crate) static BACKUP_EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_backup_handler_epoch_ending_epoch",
        "Current epoch returned in an epoch ending backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_backup_handler_transaction_version",
        "Current version returned in a transaction backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_STATE_SNAPSHOT_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_backup_handler_state_snapshot_version",
        "Version of requested state snapshot backup."
    )
    .unwrap()
});

pub(crate) static BACKUP_STATE_SNAPSHOT_LEAF_IDX: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_backup_handler_state_snapshot_leaf_index",
        "Index of current leaf index returned in a state snapshot backup."
    )
    .unwrap()
});

pub static BACKUP_TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "cedra_backup_handler_timers_seconds",
        "Various timers for performance analysis.",
        &["name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 32).unwrap(),
    )
    .unwrap()
});

pub static CONCURRENCY_GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "cedra_storage_api_concurrency",
        "Call concurrency by API.",
        &["name"]
    )
    .unwrap()
});

pub static GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!("cedra_storage_gauge", "Various gauges", &["name"]).unwrap()
});

pub static COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        // metric name
        "cedra_storage_counter",
        // metric description
        "Various counters for Cedra DB / storage.",
        // metric labels (dimensions)
        &["name"],
    )
    .unwrap()
});
