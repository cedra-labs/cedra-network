// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

#![allow(unexpected_cfgs)]

use crate::{
    backup_types::{
        state_snapshot::{
            backup::{StateSnapshotBackupController, StateSnapshotBackupOpt},
            restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        },
        transaction::{
            backup::{TransactionBackupController, TransactionBackupOpt},
            restore::{TransactionRestoreController, TransactionRestoreOpt},
        },
    },
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{
        backup_service_client::BackupServiceClient, test_utils::start_local_backup_service,
        ConcurrentDownloadsOpt, GlobalBackupOpt, GlobalRestoreOpt, GlobalRestoreOptions,
        ReplayConcurrencyLevelOpt, RocksdbOpt, TrustedWaypointOpt,
    },
};
use cedra_db::{state_restore::StateSnapshotRestoreMode, CedraDB};
use cedra_executor_test_helpers::integration_test_impl::test_execution_with_storage_impl;
use cedra_executor_types::VerifyExecutionMode;
use cedra_storage_interface::DbReader;
use cedra_temppath::TempPath;
use cedra_types::transaction::Version;
use proptest::{prelude::*, sample::Index};
use std::{convert::TryInto, sync::Arc};
use tokio::time::Duration;

#[derive(Debug)]
struct TestData {
    db: Arc<CedraDB>,
    txn_start_ver: Version,
    state_snapshot_epoch: Option<u64>,
    state_snapshot_ver: Option<u64>,
    target_ver: Version,
}

fn test_data_strategy() -> impl Strategy<Value = TestData> {
    let db = test_execution_with_storage_impl();
    let latest_ver = db.expect_synced_version();

    let latest_epoch_state = db.get_latest_epoch_state().unwrap();
    let epoch_ending_lis = db
        .get_epoch_ending_ledger_infos(0, latest_epoch_state.epoch)
        .unwrap()
        .ledger_info_with_sigs;

    any::<Index>()
        .prop_flat_map(move |state_snapshot_index| {
            let state_snapshot_epoch_li = state_snapshot_index.get(&epoch_ending_lis);
            let state_snapshot_ver = state_snapshot_epoch_li.ledger_info().version();
            let state_snapshot_epoch = state_snapshot_epoch_li.ledger_info().epoch();
            (
                0..=state_snapshot_ver,
                prop_oneof![Just(Some(state_snapshot_epoch)), Just(None)],
                Just(state_snapshot_ver),
                state_snapshot_ver..=latest_ver,
            )
        })
        .prop_map(
            move |(txn_start_ver, state_snapshot_epoch, state_snapshot_ver, target_ver)| TestData {
                db: Arc::clone(&db),
                txn_start_ver,
                state_snapshot_epoch,
                state_snapshot_ver: state_snapshot_epoch.map(|_| state_snapshot_ver),
                target_ver,
            },
        )
}

fn test_end_to_end_impl(d: TestData) {
    let tgt_db_dir = TempPath::new();
    tgt_db_dir.create_as_dir().unwrap();
    let backup_dir = TempPath::new();
    backup_dir.create_as_dir().unwrap();
    let store: Arc<dyn BackupStorage> = Arc::new(LocalFs::new(backup_dir.path().to_path_buf()));
    let (rt, port) = start_local_backup_service(Arc::clone(&d.db));
    let client = Arc::new(BackupServiceClient::new(format!(
        "http://localhost:{}",
        port
    )));
    let num_txns_to_backup = d.target_ver - d.txn_start_ver + 1;

    // Backup
    let global_backup_opt = GlobalBackupOpt {
        max_chunk_size: 2048,
        concurrent_data_requests: 2,
    };
    let state_snapshot_manifest = d.state_snapshot_epoch.map(|epoch| {
        rt.block_on(
            StateSnapshotBackupController::new(
                StateSnapshotBackupOpt { epoch },
                global_backup_opt.clone(),
                Arc::clone(&client),
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap()
    });
    let txn_manifest = rt
        .block_on(
            TransactionBackupController::new(
                TransactionBackupOpt {
                    start_version: d.txn_start_ver,
                    num_transactions: num_txns_to_backup as usize,
                },
                global_backup_opt,
                Arc::clone(&client),
                Arc::clone(&store),
            )
            .run(),
        )
        .unwrap();
    // Restore
    let global_restore_opt: GlobalRestoreOptions = GlobalRestoreOpt {
        dry_run: false,
        db_dir: Some(tgt_db_dir.path().to_path_buf()),
        target_version: Some(d.target_ver),
        trusted_waypoints: TrustedWaypointOpt::default(),
        rocksdb_opt: RocksdbOpt::default(),
        concurrent_downloads: ConcurrentDownloadsOpt::default(),
        replay_concurrency_level: ReplayConcurrencyLevelOpt::default(),
        enable_state_indices: false,
    }
    .try_into()
    .unwrap();
    if let Some(version) = d.state_snapshot_ver {
        rt.block_on(
            StateSnapshotRestoreController::new(
                StateSnapshotRestoreOpt {
                    manifest_handle: state_snapshot_manifest.unwrap(),
                    version,
                    validate_modules: false,
                    restore_mode: StateSnapshotRestoreMode::Default,
                },
                global_restore_opt.clone(),
                Arc::clone(&store),
                None, /* epoch_history */
            )
            .run(),
        )
        .unwrap()
    }
    rt.block_on(
        TransactionRestoreController::new(
            TransactionRestoreOpt {
                manifest_handle: txn_manifest,
                replay_from_version: Some(d.state_snapshot_ver.unwrap_or(Version::MAX - 1) + 1),
                kv_only_replay: Some(false),
            },
            global_restore_opt,
            store,
            None, /* epoch_history */
            VerifyExecutionMode::verify_all(),
        )
        .run(),
    )
    .unwrap();

    // Check
    let tgt_db = CedraDB::new_readonly_for_test(&tgt_db_dir);
    assert_eq!(
        d.db.get_transactions(
            d.txn_start_ver,
            num_txns_to_backup,
            d.target_ver,
            true /* fetch_events */
        )
        .unwrap(),
        tgt_db
            .get_transactions(
                d.txn_start_ver,
                num_txns_to_backup,
                d.target_ver,
                true /* fetch_events */
            )
            .unwrap()
    );
    if let Some(state_snapshot_ver) = d.state_snapshot_ver {
        let first_replayed = state_snapshot_ver + 1;
        let num_replayed = d.target_ver - state_snapshot_ver;
        // Events recreated:
        assert_eq!(
            d.db.get_transactions(first_replayed, num_replayed, d.target_ver, true)
                .unwrap(),
            tgt_db
                .get_transactions(first_replayed, num_replayed, d.target_ver, true)
                .unwrap()
        );
    };

    rt.shutdown_timeout(Duration::from_secs(1));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    // Ignore for now because the pruner now is going to see the version data to figure out the
    // progress, but we don't have version data before the state_snapshot_ver. As the result the
    // API will throw an error when getting the old transactions.
    // TODO(areshand): Figure out a plan for this.
    #[ignore]
    #[cfg_attr(feature = "consensus-only-perf-test", ignore)]
    fn test_end_to_end(d in test_data_strategy().no_shrink()) {
        test_end_to_end_impl(d)
    }
}
