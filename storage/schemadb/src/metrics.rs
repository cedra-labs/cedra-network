// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_metrics_core::{
    exponential_buckets, register_histogram_vec, register_int_counter_vec, HistogramVec,
    IntCounterVec,
};
use once_cell::sync::Lazy;

pub static CEDRA_SCHEMADB_SEEK_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_seek_latency_seconds",
        // metric description
        "Cedra schemadb seek latency in seconds",
        // metric labels (dimensions)
        &["cf_name", "tag"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_ITER_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_iter_latency_seconds",
        // metric description
        "Cedra schemadb iter latency in seconds",
        // metric labels (dimensions)
        &["cf_name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_ITER_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_iter_bytes",
        // metric description
        "Cedra schemadb iter size in bytes",
        // metric labels (dimensions)
        &["cf_name"]
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_GET_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_get_latency_seconds",
        // metric description
        "Cedra schemadb get latency in seconds",
        // metric labels (dimensions)
        &["cf_name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_GET_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_get_bytes",
        // metric description
        "Cedra schemadb get call returned data size in bytes",
        // metric labels (dimensions)
        &["cf_name"]
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_BATCH_COMMIT_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_batch_commit_latency_seconds",
        // metric description
        "Cedra schemadb schema batch commit latency in seconds",
        // metric labels (dimensions)
        &["db_name"],
        exponential_buckets(/*start=*/ 1e-3, /*factor=*/ 2.0, /*count=*/ 20).unwrap(),
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_BATCH_COMMIT_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_batch_commit_bytes",
        // metric description
        "Cedra schemadb schema batch commit size in bytes",
        // metric labels (dimensions)
        &["db_name"]
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_PUT_BYTES_SAMPLED: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "cedra_schemadb_put_bytes_sampled",
        // metric description
        "Cedra schemadb put call puts data size in bytes (sampled)",
        // metric labels (dimensions)
        &["cf_name"]
    )
    .unwrap()
});

pub static CEDRA_SCHEMADB_DELETES_SAMPLED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "cedra_storage_deletes_sampled",
        "Cedra storage delete calls (sampled)",
        &["cf_name"]
    )
    .unwrap()
});

pub static TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "cedra_schema_db_timer_seconds",
        "Various timers for performance analysis.",
        &["name", "sub_name"],
        exponential_buckets(/*start=*/ 1e-9, /*factor=*/ 2.0, /*count=*/ 32).unwrap(),
    )
    .unwrap()
});
