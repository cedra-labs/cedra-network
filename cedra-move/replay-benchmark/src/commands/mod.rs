// Copyright (c) Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_logger::{Level, Logger};
use cedra_move_debugger::cedra_debugger::CedraDebugger;
use cedra_push_metrics::MetricsPusher;
use cedra_rest_client::{CedraBaseUrl, Client};
pub use benchmark::BenchmarkCommand;
use clap::Parser;
pub use diff::DiffCommand;
pub use download::DownloadCommand;
pub use initialize::InitializeCommand;
use url::Url;

mod benchmark;
mod diff;
mod download;
mod initialize;

pub(crate) fn init_logger_and_metrics(log_level: Level) {
    let mut logger = Logger::new();
    logger.level(log_level);
    logger.init();

    let _mp = MetricsPusher::start(vec![]);
}

pub(crate) fn build_debugger(
    rest_endpoint: String,
    api_key: Option<String>,
) -> anyhow::Result<CedraDebugger> {
    let builder = Client::builder(CedraBaseUrl::Custom(Url::parse(&rest_endpoint)?));
    let client = if let Some(api_key) = api_key {
        builder.api_key(&api_key)?.build()
    } else {
        builder.build()
    };
    CedraDebugger::rest_client(client)
}

#[derive(Parser)]
pub struct RestAPI {
    #[clap(
        long,
        help = "Fullnode's REST API query endpoint, e.g., https://api.mainnet.cedralabs.com/v1 \
                for mainnet"
    )]
    rest_endpoint: String,

    #[clap(
        long,
        help = "Optional API key to increase HTTP request rate limit quota"
    )]
    api_key: Option<String>,
}
