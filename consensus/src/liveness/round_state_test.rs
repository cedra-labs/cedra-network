// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    liveness::round_state::{
        ExponentialTimeInterval, NewRoundEvent, NewRoundReason, RoundState, RoundTimeInterval,
    },
    util::mock_time_service::SimulatedTimeService,
};
use cedra_consensus_types::{
    common::Round,
    quorum_cert::QuorumCert,
    round_timeout::RoundTimeoutReason,
    sync_info::SyncInfo,
    timeout_2chain::{TwoChainTimeout, TwoChainTimeoutCertificate},
    vote_data::VoteData,
};
use cedra_crypto::HashValue;
use cedra_types::{
    aggregate_signature::AggregateSignature,
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::random_validator_verifier,
};
use futures::StreamExt;
use std::{sync::Arc, time::Duration};

#[test]
fn test_round_time_interval() {
    let interval = ExponentialTimeInterval::new(Duration::from_millis(3000), 1.5, 2);
    assert_eq!(3000, interval.get_round_duration(0).as_millis());
    assert_eq!(4500, interval.get_round_duration(1).as_millis());
    assert_eq!(
        6750, /* 4500*1.5 */
        interval.get_round_duration(2).as_millis()
    );
    // Test that there is no integer overflow
    assert_eq!(6750, interval.get_round_duration(1000).as_millis());
}

#[tokio::test]
/// Verify that RoundState properly outputs local timeout events upon timeout
async fn test_basic_timeout() {
    let (_, verifier) = random_validator_verifier(1, None, false);
    let (mut pm, mut timeout_rx) = make_round_state();

    // jump start the round_state
    pm.process_certificates(generate_sync_info(Some(0), None, None), &verifier);
    for _ in 0..2 {
        let round = timeout_rx.next().await.unwrap();
        // Here we just test timeout send retry,
        // round for timeout is not changed as no timeout certificate was gathered at this point
        assert_eq!(1, round);
        pm.process_local_timeout(round);
    }
}

#[test]
fn test_round_event_generation() {
    let (_, verifier) = random_validator_verifier(1, None, false);
    let (mut pm, _) = make_round_state();
    // Happy path with new QC
    expect_qc(
        2,
        pm.process_certificates(generate_sync_info(Some(1), None, None), &verifier),
    );
    // Old QC does not generate anything
    assert!(pm
        .process_certificates(generate_sync_info(Some(1), None, None), &verifier)
        .is_none());
    // A TC for a higher round
    expect_timeout(
        3,
        pm.process_certificates(generate_sync_info(None, Some(2), None), &verifier),
    );
    // In case both QC and TC are present choose the one with the higher value
    expect_timeout(
        4,
        pm.process_certificates(generate_sync_info(Some(2), Some(3), None), &verifier),
    );
    // In case both QC and TC are present with the same value, choose QC
    expect_qc(
        5,
        pm.process_certificates(generate_sync_info(Some(4), Some(4), None), &verifier),
    );
}

fn make_round_state() -> (RoundState, cedra_channels::Receiver<Round>) {
    let time_interval = Box::new(ExponentialTimeInterval::fixed(Duration::from_millis(2)));
    let simulated_time = SimulatedTimeService::auto_advance_until(Duration::from_millis(4));
    let (timeout_tx, timeout_rx) = cedra_channels::new_test(1_024);
    (
        RoundState::new(time_interval, Arc::new(simulated_time), timeout_tx),
        timeout_rx,
    )
}

fn expect_qc(round: Round, event: Option<NewRoundEvent>) {
    let event = event.unwrap();
    assert_eq!(round, event.round);
    assert_eq!(event.reason, NewRoundReason::QCReady);
}

fn expect_timeout(round: Round, event: Option<NewRoundEvent>) {
    let event = event.unwrap();
    assert_eq!(round, event.round);
    assert_eq!(
        event.reason,
        NewRoundReason::Timeout(RoundTimeoutReason::Unknown)
    );
}

fn generate_sync_info(
    quorum_round: Option<Round>,
    timeout_round: Option<Round>,
    commit_round: Option<Round>,
) -> SyncInfo {
    let quorum_round = quorum_round.unwrap_or(0);
    let timeout_round = timeout_round.unwrap_or(0);
    let commit_round = commit_round.unwrap_or(0);
    let commit_block = BlockInfo::new(
        1,
        commit_round,
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        None,
    );
    let ledger_info = LedgerInfoWithSignatures::new(
        LedgerInfo::new(commit_block, HashValue::zero()),
        AggregateSignature::empty(),
    );
    let quorum_cert = QuorumCert::new(
        VoteData::new(
            BlockInfo::new(
                1,
                quorum_round,
                HashValue::zero(),
                HashValue::zero(),
                0,
                0,
                None,
            ),
            BlockInfo::empty(),
        ),
        ledger_info,
    );
    let commit_cert = quorum_cert.into_wrapped_ledger_info();
    let tc = TwoChainTimeoutCertificate::new(TwoChainTimeout::new(
        1,
        timeout_round,
        quorum_cert.clone(),
    ));
    SyncInfo::new(quorum_cert, commit_cert, Some(tc))
}
