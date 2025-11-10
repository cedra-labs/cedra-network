use crate::common::types::CliResult;
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ExecuteDeposit {
    /// Profile that signs (must be the multisig executor)
    #[arg(long, default_value = "multisig")]
    pub profile: String,

    /// Framework package address (e.g., 0x1)
    #[arg(long, value_name = "0x....")]
    pub framework: String,

    /// L1 token key (20 bytes) as hex (with or without 0x)
    #[arg(long, value_name = "0x<40-hex>")]
    pub l1_token: String,

    /// Receiver on L2
    #[arg(long, value_name = "0x....")]
    pub to: String,

    /// Amount to mint
    #[arg(long)]
    pub amount: u64,

    /// Deposit nonce (must be unique)
    #[arg(long)]
    pub nonce: u64,

    /// L1 tx hash bytes, hex (with/without 0x)
    #[arg(long, value_name = "0x<hex>")]
    pub tx_hash: String,
}

impl ExecuteDeposit {
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
        let func = format!("{}::bridge::execute_deposit", self.framework);

        let l1 = Self::normalize_hex(&self.l1_token);
        Self::ensure_20_bytes_hex(&l1, "l1_token")?;
        let txh = Self::normalize_hex(&self.tx_hash);

        // Signature:
        // execute_deposit(multisig, l1_token: vector<u8>, to: address, amount: u64, nonce: u64, eth_tx_hash: vector<u8>)
        let status = Command::new(cli)
            .arg("move").arg("run")
            .arg("--profile").arg(&self.profile)
            .arg("--function-id").arg(func)
            .arg("--assume-yes")
            .arg("--args").arg(format!("hex:{}", l1))
            .arg("--args").arg(format!("address:{}", self.to))
            .arg("--args").arg(format!("u64:{}", self.amount))
            .arg("--args").arg(format!("u64:{}", self.nonce))
            .arg("--args").arg(format!("hex:{}", txh))
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("execute_deposit failed".to_string());
        }

        Ok("ok".to_string())
    }
}