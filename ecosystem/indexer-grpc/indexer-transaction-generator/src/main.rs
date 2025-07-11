// Copyright (c) Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use cedra_indexer_transaction_generator::config::IndexerCliArgs;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command line arguments.
    let args = IndexerCliArgs::parse();
    args.run().await
}
