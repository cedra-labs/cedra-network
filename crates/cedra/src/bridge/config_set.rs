use crate::common::types::{
    CliCommand, CliConfig, CliError, CliTypedResult, ConfigSearchMode, ProfileOptions, RestOptions,
};
use async_trait::async_trait;
use clap::Parser;
use serde_json::{json, Value};

#[derive(Debug, Parser)]
pub struct ConfigSet {
    pub key: String,

    pub value: String,

    #[clap(flatten)]
    pub(crate) rest_options: RestOptions,
    #[clap(flatten)]
    pub(crate) profile_options: ProfileOptions,
}

#[async_trait]
impl CliCommand<serde_json::Value> for ConfigSet {
    fn command_name(&self) -> &'static str {
        "bridge::config_set"
    }

    async fn execute(self) -> CliTypedResult<Value> {
        let _maybe_profile = CliConfig::load_profile(
            self.profile_options.profile_name(),
            ConfigSearchMode::CurrentDirAndParents,
        )?;

        Ok(json!({
            "ok": true,
            "message": "Config updated (stub). Persist wiring TODO.",
            "key": self.key,
            "value": self.value
        }))
    }
}
