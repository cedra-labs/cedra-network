use crate::common::types::{CliCommand, CliResult};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum BridgeTool {
}

impl BridgeTool {
    pub async fn execute(self) -> CliResult {
        use BridgeTool::*;
        match self {
        }
    }
}