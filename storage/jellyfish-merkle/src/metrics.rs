// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_metrics_core::{register_int_counter, register_int_gauge, IntCounter, IntGauge};
use once_cell::sync::Lazy;

pub static CEDRA_JELLYFISH_LEAF_ENCODED_BYTES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "cedra_jellyfish_leaf_encoded_bytes",
        "Cedra jellyfish leaf encoded bytes in total"
    )
    .unwrap()
});

pub static CEDRA_JELLYFISH_INTERNAL_ENCODED_BYTES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "cedra_jellyfish_internal_encoded_bytes",
        "Cedra jellyfish total internal nodes encoded in bytes"
    )
    .unwrap()
});

pub static CEDRA_JELLYFISH_LEAF_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "cedra_jellyfish_leaf_count",
        "Total number of leaves in the latest JMT."
    )
    .unwrap()
});

pub static CEDRA_JELLYFISH_LEAF_DELETION_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "cedra_jellyfish_leaf_deletion_count",
        "The number of deletions happened in JMT."
    )
    .unwrap()
});
