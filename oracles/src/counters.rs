// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_metrics_core::{register_histogram_vec, register_int_gauge, HistogramVec, IntGauge};
use std::sync::LazyLock;

/// Count of the pending messages sent to itself in the channel
pub static PENDING_SELF_MESSAGES: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!(
        "cedra_oracle_pending_self_messages",
        "Count of the pending messages sent to itself in the channel"
    )
    .unwrap()
});
