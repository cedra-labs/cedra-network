// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::db_access::DbAccessUtil;
use anyhow::Result;
use cedra_storage_interface::{
    state_store::state_view::db_state_view::LatestDbStateCheckpointView, DbReaderWriter,
};
use cedra_transaction_generator_lib::{CounterState, ReliableTransactionSubmitter};
use cedra_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    state_store::MoveResourceExt,
    transaction::{SignedTransaction, Transaction},
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, mpsc},
    time::Duration,
};

pub struct DbReliableTransactionSubmitter {
    pub db: DbReaderWriter,
    pub block_sender: mpsc::SyncSender<Vec<Transaction>>,
}

#[async_trait]
impl ReliableTransactionSubmitter for DbReliableTransactionSubmitter {
    async fn get_account_balance(&self, account_address: AccountAddress) -> Result<u64> {
        let db_state_view = self.db.reader.latest_state_checkpoint_view().unwrap();
        DbAccessUtil::get_fungible_store(&account_address, &db_state_view)
            .map(|fungible_store| fungible_store.balance())
    }

    async fn query_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        let db_state_view = self.db.reader.latest_state_checkpoint_view().unwrap();
        Ok(
            AccountResource::fetch_move_resource(&db_state_view, &address)
                .unwrap()
                .map(|account| account.sequence_number())
                .unwrap_or(0),
        )
        //.context("account doesn't exist")
    }

    async fn execute_transactions_with_counter(
        &self,
        txns: &[SignedTransaction],
        _state: &CounterState,
    ) -> Result<()> {
        self.block_sender.send(
            txns.iter()
                .map(|t| Transaction::UserTransaction(t.clone()))
                .collect(),
        )?;

        for txn in txns {
            // Pipeline commit makes sure all initialization transactions
            // get committed succesfully on-chain
            while txn.sequence_number() >= self.query_sequence_number(txn.sender()).await? {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        Ok(())
    }

    fn create_counter_state(&self) -> CounterState {
        CounterState {
            submit_failures: vec![AtomicUsize::new(0)],
            wait_failures: vec![AtomicUsize::new(0)],
            successes: AtomicUsize::new(0),
            by_client: HashMap::new(),
        }
    }
}
