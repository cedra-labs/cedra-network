// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use cedra_move_debugger::cedra_debugger::CedraDebugger;
use cedra_rest_client::Client;
use cedra_types::transaction::Transaction;
use cedra_vm::CedraVM;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Subcommand)]
pub enum Target {
    /// Use full node's rest api as query endpoint.
    Rest { endpoint: String },
    /// Use a local db instance to serve as query endpoint.
    DB { path: PathBuf },
}

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    target: Target,

    #[clap(long)]
    version: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse the commandline args
    let args = Args::parse();
    let version = args.version;

    // Initialize the debugger
    cedra_logger::Logger::new().init();
    CedraVM::set_concurrency_level_once(1);

    let debugger = match args.target {
        Target::Rest { endpoint } => {
            CedraDebugger::rest_client(Client::new(Url::parse(&endpoint)?))?
        },
        Target::DB { path } => CedraDebugger::db(path)?,
    };

    // Execute the transaction w/ the gas profiler
    let (txn, _txn_info) = debugger
        .get_committed_transaction_at_version(version)
        .await?;

    let txn = match txn {
        Transaction::UserTransaction(txn) => txn,
        _ => bail!("not a user transaction"),
    };

    let (_status, output, gas_log) =
        debugger.execute_transaction_at_version_with_gas_profiler(version, txn)?;

    let txn_output =
        output.try_materialize_into_transaction_output(&debugger.state_view_at_version(version))?;

    // Show results to the user
    println!("{:#?}", txn_output);

    let report_path = Path::new("gas-profiling").join(format!("txn-{}", version));
    gas_log.generate_html_report(
        &report_path,
        format!("Gas Report - Transaction {}", version),
    )?;

    println!("Gas profiling report saved to {}.", report_path.display());

    Ok(())
}
