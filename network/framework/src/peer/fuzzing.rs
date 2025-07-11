// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants,
    peer::Peer,
    protocols::wire::{
        handshake::v1::{MessagingProtocolVersion, ProtocolIdSet},
        messaging::v1::{MultiplexMessage, MultiplexMessageSink},
    },
    testutils::fake_socket::ReadOnlyTestSocketVec,
    transport::{Connection, ConnectionId, ConnectionMetadata},
};
use cedra_channels::{cedra_channel, message_queues::QueueStyle};
use cedra_config::{config::PeerRole, network_id::NetworkContext};
use cedra_memsocket::MemorySocket;
use cedra_netcore::transport::ConnectionOrigin;
use cedra_proptest_helpers::ValueGenerator;
use cedra_time_service::TimeService;
use cedra_types::{network_address::NetworkAddress, PeerId};
use futures::{executor::block_on, future, io::AsyncReadExt, sink::SinkExt, stream::StreamExt};
use proptest::{arbitrary::any, collection::vec};
use std::{collections::HashMap, sync::Arc, time::Duration};

/// Generate a sequence of `MultiplexMessage`, bcs serialize them, and write them
/// out to a buffer using our length-prefixed message codec.
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    let network_msgs = gen.generate(vec(any::<MultiplexMessage>(), 1..20));

    let (write_socket, mut read_socket) = MemorySocket::new_pair();
    let mut writer = MultiplexMessageSink::new(write_socket, constants::MAX_FRAME_SIZE);

    // Write the `MultiplexMessage`s to a fake socket
    let f_send = async move {
        for network_msg in &network_msgs {
            writer.send(network_msg).await.unwrap();
        }
    };
    // Read the serialized `MultiplexMessage`s from the fake socket
    let f_recv = async move {
        let mut buf = Vec::new();
        read_socket.read_to_end(&mut buf).await.unwrap();
        buf
    };

    let (_, buf) = block_on(future::join(f_send, f_recv));
    buf
}

/// Fuzz the `Peer` actor's inbound message handling.
///
/// For each fuzzer iteration, we spin up a new `Peer` actor and pipe the raw
/// fuzzer data into it. This mostly tests that the `Peer` inbound message handling
/// doesn't panic or leak memory when reading, deserializing, and handling messages
/// from remote peers.
pub fn fuzz(data: &[u8]) {
    // Use the basic single-threaded runtime, since our current tokio version has
    // a chance to leak memory and/or thread handles when using the threaded
    // runtime and sometimes blocks when trying to shutdown the runtime.
    //
    // https://github.com/tokio-rs/tokio/pull/2649
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let executor = rt.handle().clone();

    // We want to choose a constant peer id for _our_ peer id, since we will
    // generate unbounded metrics otherwise and OOM during fuzzing.
    let peer_id = PeerId::ZERO;
    // However, we want to choose a random _remote_ peer id to ensure we _don't_
    // have metrics logging the remote peer id (which would eventually OOM in
    // production for public-facing nodes).
    let remote_peer_id = PeerId::random();

    // Mock data
    let network_context = NetworkContext::mock_with_peer_id(peer_id);
    let socket = ReadOnlyTestSocketVec::new(data.to_vec());
    let metadata = ConnectionMetadata::new(
        remote_peer_id,
        ConnectionId::from(123),
        NetworkAddress::mock(),
        ConnectionOrigin::Inbound,
        MessagingProtocolVersion::V1,
        ProtocolIdSet::all_known(),
        PeerRole::Unknown,
    );
    let connection = Connection { socket, metadata };

    let (connection_notifs_tx, connection_notifs_rx) = cedra_channels::new_test(8);
    let channel_size = 8;

    let (peer_reqs_tx, peer_reqs_rx) = cedra_channel::new(QueueStyle::FIFO, channel_size, None);
    let upstream_handlers = Arc::new(HashMap::new());

    // Spin up a new `Peer` actor
    let peer = Peer::new(
        network_context,
        executor.clone(),
        TimeService::mock(),
        connection,
        connection_notifs_tx,
        peer_reqs_rx,
        upstream_handlers,
        Duration::from_millis(constants::INBOUND_RPC_TIMEOUT_MS),
        constants::MAX_CONCURRENT_INBOUND_RPCS,
        constants::MAX_CONCURRENT_OUTBOUND_RPCS,
        constants::MAX_FRAME_SIZE,
        constants::MAX_MESSAGE_SIZE,
    );
    executor.spawn(peer.start());

    rt.block_on(async move {
        // Wait for "remote" to disconnect (we read all data and socket read
        // returns EOF), we read a disconnect request, or we fail to deserialize
        // something.
        connection_notifs_rx.collect::<Vec<_>>().await;

        // ACK the "remote" d/c and drop our handle to the Peer actor. Then wait
        // for all network notifs to drain out and finish.
        drop(peer_reqs_tx);
    });
}

#[test]
fn test_peer_fuzzers() {
    let mut value_gen = ValueGenerator::deterministic();
    for _ in 0..50 {
        let corpus = generate_corpus(&mut value_gen);
        fuzz(&corpus);
    }
}
