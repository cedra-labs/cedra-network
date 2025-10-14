use crate::common::types::{CliCommand, CliResult};
use clap::Subcommand;

pub mod config_set;
pub mod init;
pub mod status;

#[derive(Debug, Subcommand)]
pub enum BridgeTool {
    Init(init::Init),
    Status(status::Status),
    ConfigSet(config_set::ConfigSet),
}

impl BridgeTool {
    pub async fn execute(self) -> CliResult {
        match self {
            BridgeTool::Init(tool) => tool.execute_serialized().await,
            BridgeTool::Status(tool) => tool.execute_serialized().await,
            BridgeTool::ConfigSet(tool) => tool.execute_serialized().await,
        }
    }
}
