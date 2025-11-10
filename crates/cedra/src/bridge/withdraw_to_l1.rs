use crate::common::types::CliResult;
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct WithdrawToL1 {
    /// Profile of the withdrawing user
    #[arg(long, default_value = "user")]
    pub profile: String,

    /// Framework package address (e.g. 0x1)
    #[arg(long, value_name = "0x....")]
    pub framework: String,

    /// L1 token key (20 bytes) as hex (with or without 0x)
    #[arg(long, value_name = "0x<40-hex>")]
    pub l1_token: String,

    /// 20-byte ETH recipient, hex (with or without 0x)
    #[arg(long, value_name = "0x<40-hex>")]
    pub recipient: String,

    /// Amount to withdraw (u64)
    #[arg(long)]
    pub amount: u64,

    /// Withdrawal nonce (u64) approved by multisig
    #[arg(long)]
    pub nonce: u64,
}

impl WithdrawToL1 {
    fn normalize_hex(input: &str) -> String {
        if input.starts_with("0x") { input.to_string() } else { format!("0x{}", input) }
    }

    fn ensure_20_bytes_hex(h: &str, field: &str) -> Result<(), String> {
        let s = h.strip_prefix("0x").unwrap_or(h);
        if s.len() != 40 { return Err(format!("{field} must be 20 bytes (40 hex chars)")); }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) { return Err(format!("{field} must be hex")); }
        Ok(())
    }

    pub async fn execute_serialized(self) -> CliResult {
        let cli = std::env::var("CLI").unwrap_or_else(|_| "cedra".to_string());
        let func = format!("{}::bridge::withdraw_to_l1", self.framework);

        let l1 = Self::normalize_hex(&self.l1_token);
        let recip = Self::normalize_hex(&self.recipient);
        Self::ensure_20_bytes_hex(&l1, "l1_token")?;
        Self::ensure_20_bytes_hex(&recip, "recipient")?;

        // Move signature:
        // withdraw_to_l1(user, l1_token: vector<u8>, eth_recipient: vector<u8>,
        //                amount: u64, nonce: u64)
        let status = Command::new(cli)
            .arg("move").arg("run")
            .arg("--profile").arg(&self.profile)
            .arg("--function-id").arg(func)
            .arg("--assume-yes")
            .arg("--args").arg(format!("hex:{}", l1))
            .arg("--args").arg(format!("hex:{}", recip))
            .arg("--args").arg(format!("u64:{}", self.amount))
            .arg("--args").arg(format!("u64:{}", self.nonce))
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("withdraw_to_l1 failed".to_string());
        }

        Ok("ok".to_string())
    }
}