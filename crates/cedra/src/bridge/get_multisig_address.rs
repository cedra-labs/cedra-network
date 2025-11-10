use crate::common::types::CliResult;
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct AdminMultisig {
    /// Framework package address (e.g., 0x1)
    #[arg(long, value_name = "0x....")]
    pub framework: String,

    /// Profile to use for the view
    #[arg(long, default_value = "user")]
    pub profile: String,
}

impl AdminMultisig {
    pub async fn execute_serialized(self) -> CliResult {
        let cli = std::env::var("CLI").unwrap_or_else(|_| "cedra".to_string());
        let func = format!("{}::bridge::admin_multisig", self.framework);

        // Run the view and capture output
        let out = Command::new(cli)
            .arg("move").arg("view")
            .arg("--function-id").arg(func)
            .arg("--profile").arg(&self.profile)
            .output()
            .map_err(|e| e.to_string())?;

        if !out.status.success() {
            return Err(format!(
                "view admin_multisig failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }

        // Return stdout as the successful result
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        Ok(stdout)
    }
}