// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use super::{cedra::CedraUpdateTool, revela::RevelaUpdateTool};
use crate::{
    common::types::{CliCommand, CliResult},
    update::{
        move_mutation_test::MutationTestUpdaterTool, movefmt::FormatterUpdateTool,
        prover_dependencies::ProverDependencyInstaller,
    },
};
use clap::Subcommand;

/// Update the CLI or other tools it depends on.
#[derive(Subcommand)]
pub enum UpdateTool {
    Cedra(CedraUpdateTool),
    Revela(RevelaUpdateTool),
    Movefmt(FormatterUpdateTool),
    MoveMutationTest(MutationTestUpdaterTool),
    ProverDependencies(ProverDependencyInstaller),
}

impl UpdateTool {
    pub async fn execute(self) -> CliResult {
        match self {
            UpdateTool::Cedra(tool) => tool.execute_serialized().await,
            UpdateTool::Revela(tool) => tool.execute_serialized().await,
            UpdateTool::Movefmt(tool) => tool.execute_serialized().await,
            UpdateTool::MoveMutationTest(tool) => tool.execute_serialized().await,
            UpdateTool::ProverDependencies(tool) => tool.execute_serialized().await,
        }
    }
}
