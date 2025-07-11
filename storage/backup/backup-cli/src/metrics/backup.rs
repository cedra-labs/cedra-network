// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_metrics_core::{register_int_counter_vec, IntCounterVec};
use cedra_push_metrics::{
    exponential_buckets, register_histogram_vec, register_int_gauge, HistogramVec, IntGauge,
};
use once_cell::sync::Lazy;

pub static HEARTBEAT_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_db_backup_coordinator_heartbeat_timestamp_s",
        "Timestamp when the backup coordinator successfully updates state from the backup service."
    )
    .unwrap()
});

pub static EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_db_backup_coordinator_epoch_ending_epoch",
        "Epoch of the latest epoch ending backed up."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_db_backup_coordinator_state_snapshot_epoch",
        "The epoch at the end of which the latest state snapshot was taken."
    )
    .unwrap()
});

pub static TRANSACTION_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_db_backup_coordinator_transaction_version",
        "Version of the latest transaction backed up."
    )
    .unwrap()
});

pub static COMPACTED_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_db_backup_coordinator_compacted_version",
        "Version of the latest transaction metadata compacted."
    )
    .unwrap()
});

pub static BACKUP_TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "cedra_db_backup_timers_seconds",
        "Various timers for performance analysis.",
        &["name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 32).unwrap(),
    )
    .unwrap()
});

pub static THROUGHPUT_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "cedra_db_backup_received_bytes",
        "Backup controller throughput in bytes.",
        &["endpoint"]
    )
    .unwrap()
});
