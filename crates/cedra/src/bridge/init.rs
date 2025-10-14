use crate::common::types::{
    CliCommand, CliConfig, CliError, CliTypedResult, ConfigSearchMode, ProfileOptions, RestOptions,
};
use async_trait::async_trait;
use clap::Parser;
use serde_json::json;

#[derive(Debug, Parser)]
pub struct Init {
    #[clap(long)]
    pub network: String,

    #[clap(long)]
    pub rpc: Option<String>,

    #[clap(long)]
    pub chain_id: Option<u64>,

    #[clap(long)]
    pub wrapper: Option<String>,

    #[clap(long)]
    pub yes: bool,

    #[clap(flatten)]
    pub(crate) rest_options: RestOptions,
    #[clap(flatten)]
    pub(crate) profile_options: ProfileOptions,
}

#[async_trait]
impl CliCommand<serde_json::Value> for Init {
    fn command_name(&self) -> &'static str {
        "bridge::init"
    }

    async fn execute(self) -> CliTypedResult<serde_json::Value> {
        let _maybe_profile = CliConfig::load_profile(
            self.profile_options.profile_name(),
            ConfigSearchMode::CurrentDirAndParents,
        )?;

        Ok(json!({
            "ok": true,
            "message": "Bridge initialized (stub). Persist wiring TODO.",
            "network": self.network,
            "rpc": self.rpc,
            "chain_id": self.chain_id,
            "wrapper": self.wrapper,
            "yes": self.yes
        }))
    }
}
