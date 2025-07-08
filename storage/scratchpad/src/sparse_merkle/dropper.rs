// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use cedra_drop_helper::async_concurrent_dropper::AsyncConcurrentDropper;
use once_cell::sync::Lazy;

pub static SUBTREE_DROPPER: Lazy<AsyncConcurrentDropper> =
    Lazy::new(|| AsyncConcurrentDropper::new("smt_subtree", 32, 8));
