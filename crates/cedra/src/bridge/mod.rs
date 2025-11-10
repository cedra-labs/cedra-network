use crate::common::types::{CliCommand, CliResult};
use clap::Subcommand;

pub mod get_multisig_address;
pub mod nonce_used;
pub mod execute_deposit;
pub mod approve_withdrawal;
pub mod withdraw_to_l1;


#[derive(Debug, Subcommand)]
pub enum BridgeTool {
    ApproveWithdrawal(approve_withdrawal::ApproveWithdrawal),
    WithdrawToL1(withdraw_to_l1::WithdrawToL1),
    AdminMultisig(get_multisig_address::AdminMultisig),
    NonceUsed(nonce_used::NonceUsed),
    ExecuteDeposit(execute_deposit::ExecuteDeposit),
}

impl BridgeTool {
    pub async fn execute(self) -> CliResult {
        use BridgeTool::*;
        match self {
            ApproveWithdrawal(t)         => t.execute_serialized().await,
            WithdrawToL1(t)        => t.execute_serialized().await,
            AdminMultisig(t)          => t.execute_serialized().await,
            NonceUsed(t)         => t.execute_serialized().await,
            ExecuteDeposit(t)   => t.execute_serialized().await,
        }
    }
}