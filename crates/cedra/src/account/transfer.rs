// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::{CliCommand, CliTypedResult, TransactionOptions};
use cedra_cached_packages::cedra_stdlib;
use cedra_rest_client::{
    cedra_api_types::{HashValue, WriteResource, WriteSetChange},
    Transaction,
};
use cedra_types::account_address::AccountAddress;
use async_trait::async_trait;
use clap::Parser;
use serde::Serialize;
use std::collections::BTreeMap;

// TODO: Add ability to transfer non-Cedra coins
// TODO: Add ability to not create account by default
/// Transfer Cedra between accounts
///
#[derive(Debug, Parser)]
pub struct TransferCoins {
    /// Address of account to send Cedra to
    #[clap(long, value_parser = crate::common::types::load_account_arg)]
    pub(crate) account: AccountAddress,

    /// Amount of Octas (10^-8 Cedra) to transfer
    #[clap(long)]
    pub(crate) amount: u64,

    #[clap(flatten)]
    pub(crate) txn_options: TransactionOptions,
}

#[async_trait]
impl CliCommand<TransferSummary> for TransferCoins {
    fn command_name(&self) -> &'static str {
        "TransferCoins"
    }

    async fn execute(self) -> CliTypedResult<TransferSummary> {
        self.txn_options
            .submit_transaction(cedra_stdlib::cedra_account_transfer(
                self.account,
                self.amount,
            ))
            .await
            .map(TransferSummary::from)
    }
}

const SUPPORTED_COINS: [&str; 1] = ["0x1::coin::CoinStore<0x1::cedra_coin::CedraCoin>"];

/// A shortened transaction output
#[derive(Clone, Debug, Serialize)]
pub struct TransferSummary {
    pub gas_unit_price: u64,
    pub gas_used: u64,
    pub balance_changes: BTreeMap<AccountAddress, serde_json::Value>,
    pub sender: AccountAddress,
    pub success: bool,
    pub version: u64,
    pub vm_status: String,
    pub transaction_hash: HashValue,
}

impl TransferSummary {
    pub fn octa_spent(&self) -> u64 {
        self.gas_unit_price * self.gas_used
    }
}

impl From<Transaction> for TransferSummary {
    fn from(transaction: Transaction) -> Self {
        if let Transaction::UserTransaction(txn) = transaction {
            let vm_status = txn.info.vm_status;
            let success = txn.info.success;
            let sender = *txn.request.sender.inner();
            let gas_unit_price = txn.request.gas_unit_price.0;
            let gas_used = txn.info.gas_used.0;
            let transaction_hash = txn.info.hash;
            let version = txn.info.version.0;
            let balance_changes = txn
                .info
                .changes
                .into_iter()
                .filter_map(|change| match change {
                    WriteSetChange::WriteResource(WriteResource { address, data, .. }) => {
                        if SUPPORTED_COINS.contains(&data.typ.to_string().as_str()) {
                            Some((
                                *address.inner(),
                                serde_json::to_value(data.data).unwrap_or_default(),
                            ))
                        } else {
                            None
                        }
                    },
                    _ => None,
                })
                .collect();

            TransferSummary {
                gas_unit_price,
                gas_used,
                balance_changes,
                sender,
                success,
                version,
                vm_status,
                transaction_hash,
            }
        } else {
            panic!("Can't call From<Transaction> for a non UserTransaction")
        }
    }
}
