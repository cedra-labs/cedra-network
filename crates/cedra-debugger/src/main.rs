// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use cedra_debugger::Cmd;
use cedra_logger::{Level, Logger};
use cedra_push_metrics::MetricsPusher;
use clap::Parser;

#[cfg(unix)]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    Logger::new().level(Level::Info).init();
    let _mp = MetricsPusher::start(vec![]);

    Cmd::parse().run().await
}
