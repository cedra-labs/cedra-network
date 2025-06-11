// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_cargo_cli::{CedraCargoCommand, SelectedPackageArgs};
use clap::Parser;
use std::process::exit;

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    #[command(name = "x")]
    CedraCargoTool(CedraCargoToolArgs),
}

#[derive(Parser)]
struct CedraCargoToolArgs {
    #[command(subcommand)]
    cmd: CedraCargoCommand,
    #[command(flatten)]
    package_args: SelectedPackageArgs,
}

fn main() {
    let CargoCli::CedraCargoTool(args) = CargoCli::parse();
    let CedraCargoToolArgs { cmd, package_args } = args;
    let result = cmd.execute(&package_args);

    // At this point, we'll want to print and determine whether to exit for an error code
    match result {
        Ok(_) => {},
        Err(inner) => {
            println!("{}", inner);
            exit(1);
        },
    }
}
