// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::CedraDB;
use anyhow::anyhow;
use cedra_config::config::{NodeConfig, StorageDirPaths};
use cedra_crypto::HashValue;
use cedra_db_indexer::db_indexer::InternalIndexerDB;
use cedra_infallible::RwLock;
use cedra_storage_interface::{
    chunk_to_commit::ChunkToCommit, DbReader, DbWriter, Result, StateSnapshotReceiver,
};
use cedra_types::{
    ledger_info::LedgerInfoWithSignatures,
    state_store::{state_key::StateKey, state_value::StateValue},
    transaction::{TransactionOutputListWithProof, Version},
};
use either::Either;
use std::{sync::Arc, time::Instant};
use tokio::sync::watch::Sender;
pub const SECONDARY_DB_DIR: &str = "fast_sync_secondary";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FastSyncStatus {
    UNKNOWN,
    STARTED,
    FINISHED,
}

/// This is a wrapper around [CedraDB] that is used to bootstrap the node for fast sync mode
pub struct FastSyncStorageWrapper {
    // Used for storing genesis data during fast sync
    temporary_db_with_genesis: Arc<CedraDB>,
    // Used for restoring fast sync snapshot and all the read/writes afterwards
    db_for_fast_sync: Arc<CedraDB>,
    // This is for reading the fast_sync status to determine which db to use
    fast_sync_status: Arc<RwLock<FastSyncStatus>>,
}

impl FastSyncStorageWrapper {
    /// If the db is empty and configured to do fast sync, we return a FastSyncStorageWrapper
    /// Otherwise, we return CedraDB directly and the FastSyncStorageWrapper is None
    pub fn initialize_dbs(
        config: &NodeConfig,
        internal_indexer_db: Option<InternalIndexerDB>,
        update_sender: Option<Sender<(Instant, Version)>>,
    ) -> Result<Either<CedraDB, Self>> {
        let mut db_main = CedraDB::open(
            config.storage.get_dir_paths(),
            /*readonly=*/ false,
            config.storage.storage_pruner_config,
            config.storage.rocksdb_configs,
            config.storage.enable_indexer,
            config.storage.buffered_state_target_items,
            config.storage.max_num_nodes_per_lru_cache_shard,
            internal_indexer_db,
        )
        .map_err(|err| anyhow!("fast sync DB failed to open {}", err))?;
        if let Some(sender) = update_sender {
            db_main.add_version_update_subscriber(sender)?;
        }

        let mut db_dir = config.storage.dir();
        // when the db is empty and configured to do fast sync, we will create a second DB
        if config
            .state_sync
            .state_sync_driver
            .bootstrapping_mode
            .is_fast_sync()
            && (db_main
                .ledger_db
                .metadata_db()
                .get_synced_version()?
                .map_or(0, |v| v)
                == 0)
        {
            db_dir.push(SECONDARY_DB_DIR);
            let secondary_db = CedraDB::open(
                StorageDirPaths::from_path(db_dir.as_path()),
                /*readonly=*/ false,
                config.storage.storage_pruner_config,
                config.storage.rocksdb_configs,
                config.storage.enable_indexer,
                config.storage.buffered_state_target_items,
                config.storage.max_num_nodes_per_lru_cache_shard,
                None,
            )
            .map_err(|err| anyhow!("Secondary DB failed to open {}", err))?;

            Ok(Either::Right(FastSyncStorageWrapper {
                temporary_db_with_genesis: Arc::new(secondary_db),
                db_for_fast_sync: Arc::new(db_main),
                fast_sync_status: Arc::new(RwLock::new(FastSyncStatus::UNKNOWN)),
            }))
        } else {
            Ok(Either::Left(db_main))
        }
    }

    pub fn get_fast_sync_db(&self) -> Arc<CedraDB> {
        self.db_for_fast_sync.clone()
    }

    pub fn get_temporary_db_with_genesis(&self) -> Arc<CedraDB> {
        self.temporary_db_with_genesis.clone()
    }

    pub fn get_fast_sync_status(&self) -> FastSyncStatus {
        *self.fast_sync_status.read()
    }

    /// Check if the fast sync finished already
    fn is_fast_sync_bootstrap_finished(&self) -> bool {
        let status = self.get_fast_sync_status();
        status == FastSyncStatus::FINISHED
    }

    /// Check if the fast sync started already
    fn is_fast_sync_bootstrap_started(&self) -> bool {
        let status = self.get_fast_sync_status();
        status == FastSyncStatus::STARTED
    }

    pub(crate) fn get_cedra_db_read_ref(&self) -> &CedraDB {
        if self.is_fast_sync_bootstrap_finished() {
            self.db_for_fast_sync.as_ref()
        } else {
            self.temporary_db_with_genesis.as_ref()
        }
    }

    pub(crate) fn get_cedra_db_write_ref(&self) -> &CedraDB {
        if self.is_fast_sync_bootstrap_started() || self.is_fast_sync_bootstrap_finished() {
            self.db_for_fast_sync.as_ref()
        } else {
            self.temporary_db_with_genesis.as_ref()
        }
    }
}

impl DbWriter for FastSyncStorageWrapper {
    fn get_state_snapshot_receiver(
        &self,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<Box<dyn StateSnapshotReceiver<StateKey, StateValue>>> {
        *self.fast_sync_status.write() = FastSyncStatus::STARTED;
        self.get_cedra_db_write_ref()
            .get_state_snapshot_receiver(version, expected_root_hash)
    }

    fn finalize_state_snapshot(
        &self,
        version: Version,
        output_with_proof: TransactionOutputListWithProof,
        ledger_infos: &[LedgerInfoWithSignatures],
    ) -> Result<()> {
        let status = self.get_fast_sync_status();
        assert_eq!(status, FastSyncStatus::STARTED);
        self.get_cedra_db_write_ref().finalize_state_snapshot(
            version,
            output_with_proof,
            ledger_infos,
        )?;
        let mut status = self.fast_sync_status.write();
        *status = FastSyncStatus::FINISHED;
        Ok(())
    }

    fn pre_commit_ledger(&self, chunk: ChunkToCommit, sync_commit: bool) -> Result<()> {
        self.get_cedra_db_write_ref()
            .pre_commit_ledger(chunk, sync_commit)
    }

    fn commit_ledger(
        &self,
        version: Version,
        ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
        chunk_opt: Option<ChunkToCommit>,
    ) -> Result<()> {
        self.get_cedra_db_write_ref()
            .commit_ledger(version, ledger_info_with_sigs, chunk_opt)
    }
}

impl DbReader for FastSyncStorageWrapper {
    fn get_read_delegatee(&self) -> &dyn DbReader {
        self.get_cedra_db_read_ref()
    }
}
