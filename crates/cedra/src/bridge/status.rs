use crate::common::types::{
    CliCommand, CliConfig, CliError, CliTypedResult, ConfigSearchMode, ProfileOptions, RestOptions,
};
use async_trait::async_trait;
use clap::Parser;
use serde_json::json;

#[derive(Debug, Parser)]
pub struct Status {
    #[clap(flatten)]
    pub(crate) rest_options: RestOptions,
    #[clap(flatten)]
    pub(crate) profile_options: ProfileOptions,
}

#[async_trait]
impl CliCommand<serde_json::Value> for Status {
    fn command_name(&self) -> &'static str {
        "bridge::status"
    }

    async fn execute(self) -> CliTypedResult<serde_json::Value> {
        let _maybe_profile = CliConfig::load_profile(
            self.profile_options.profile_name(),
            ConfigSearchMode::CurrentDirAndParents,
        )?;

        Ok(json!({
            "network": "sepolia",
            "rpc": "https://example-rpc",
            "chain_id": 11155111,
            "wrapper": "0xDEAD...BEEF",
            "multisig": {
                "set_id": "cedra-main",
                "threshold": 2,
                "owners": ["0xOwner1...", "0xOwner2..."]
            },
            "notes": "Stub output; wire to config + RPC reads."
        }))
    }
}
