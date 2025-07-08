// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use cedra_backup_service::start_backup_service;
use cedra_config::utils::get_available_port;
use cedra_db::{db::test_helper::arb_blocks_to_commit, CedraDB};
use cedra_proptest_helpers::ValueGenerator;
use cedra_temppath::TempPath;
use cedra_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionToCommit, Version},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::runtime::Runtime;

pub fn tmp_db_empty() -> (TempPath, Arc<CedraDB>) {
    let tmpdir = TempPath::new();
    let db = Arc::new(CedraDB::new_for_test(&tmpdir));

    (tmpdir, db)
}

pub fn tmp_db_with_random_content() -> (
    TempPath,
    Arc<CedraDB>,
    Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) {
    let (tmpdir, db) = tmp_db_empty();
    let mut cur_ver: Version = 0;
    let blocks = ValueGenerator::new().generate(arb_blocks_to_commit());
    for (txns_to_commit, ledger_info_with_sigs) in &blocks {
        db.save_transactions_for_test(
            txns_to_commit,
            cur_ver, /* first_version */
            Some(ledger_info_with_sigs),
            true, /* sync_commit */
        )
        .unwrap();
        cur_ver += txns_to_commit.len() as u64;
    }

    (tmpdir, db, blocks)
}

pub fn start_local_backup_service(db: Arc<CedraDB>) -> (Runtime, u16) {
    let port = get_available_port();
    let rt = start_backup_service(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port), db);
    (rt, port)
}
