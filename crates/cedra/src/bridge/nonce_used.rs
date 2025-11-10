use crate::common::types::CliResult;
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct NonceUsed {
    /// Framework package address (e.g., 0x1)
    #[arg(long, value_name = "0x....")]
    pub framework: String,

    /// Nonce to query
    #[arg(long)]
    pub nonce: u64,

    /// Profile to use for the view
    #[arg(long, default_value = "user")]
    pub profile: String,
}

impl NonceUsed {
    pub async fn execute_serialized(self) -> CliResult {
        let cli = std::env::var("CLI").unwrap_or_else(|_| "cedra".to_string());
        let func = format!("{}::bridge::nonce_used", self.framework);

        let out = Command::new(cli)
            .arg("move").arg("view")
            .arg("--function-id").arg(func)
            .arg("--profile").arg(&self.profile)
            .arg("--args").arg(format!("u64:{}", self.nonce))
            .output()
            .map_err(|e| e.to_string())?;

        if !out.status.success() {
            return Err(format!(
                "view nonce_used failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}