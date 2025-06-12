// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use cedra_node::{utils::ERROR_MSG_BAD_FEATURE_FLAGS, CedraNodeArgs};
use clap::Parser;

#[cfg(unix)]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    // Check that we are not including any Move test natives
    cedra_vm::natives::assert_no_test_natives(ERROR_MSG_BAD_FEATURE_FLAGS);

    // Start the node
    CedraNodeArgs::parse().run()
}
