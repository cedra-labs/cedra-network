// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use cedra_metrics_core::{
    exponential_buckets, register_histogram_vec, register_int_gauge_vec, HistogramVec, IntGaugeVec,
};
use once_cell::sync::Lazy;

pub static TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "cedra_scratchpad_smt_timer_seconds",
        "Various timers for performance analysis.",
        &["name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static GENERATION: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "cedra_scratchpad_smt_generation",
        "Various generations to help debugging.",
        &["name"],
    )
    .unwrap()
});
