//Todo: here must be writed first http network interface, next rpc protocol to connect to price fetcher

// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::types::OraclesRequest;
use bytes::Bytes;
use cedra_config::network_id::{NetworkId, PeerNetworkId};
use cedra_network::{
    application::{error::Error, interface::NetworkClientInterface},
    ProtocolId,
};
use cedra_types::PeerId;
use std::{collections::HashMap, time::Duration};

pub const RPC: &[ProtocolId] = &[
    ProtocolId::DKGRpcCompressed,
    ProtocolId::DKGRpcBcs,
    ProtocolId::DKGRpcJson,
];

pub const DIRECT_SEND: &[ProtocolId] = &[
    ProtocolId::DKGDirectSendCompressed,
    ProtocolId::DKGDirectSendBcs,
    ProtocolId::DKGDirectSendJson,
];

#[derive(Clone)]
pub struct OraclesNetworkClient<NetworkClient> {
    network_client: NetworkClient,
}

impl<NetworkClient: NetworkClientInterface<OraclesRequest>> OraclesNetworkClient<NetworkClient> {
    /// Returns a new Oracles network client
    pub fn new(network_client: NetworkClient) -> Self {
        Self { network_client }
    }

    pub async fn send_rpc(
        &self,
        peer: PeerId,
        message: OraclesRequest,
        rpc_timeout: Duration,
    ) -> Result<OraclesRequest, Error> {
        let peer_network_id = self.get_peer_network_id_for_peer(peer);
        self.network_client
            .send_to_peer_rpc(message, rpc_timeout, peer_network_id)
            .await
    }

    /// Send a RPC to the destination peer
    pub async fn send_rpc_raw(
        &self,
        peer: PeerId,
        message: Bytes,
        rpc_timeout: Duration,
    ) -> Result<OraclesRequest, Error> {
        let peer_network_id = self.get_peer_network_id_for_peer(peer);
        self.network_client
            .send_to_peer_rpc_raw(message, rpc_timeout, peer_network_id)
            .await
    }

    pub fn to_bytes_by_protocol(
        &self,
        peers: Vec<PeerId>,
        message: OraclesRequest,
    ) -> anyhow::Result<HashMap<PeerId, Bytes>> {
        let peer_network_ids: Vec<PeerNetworkId> = peers
            .into_iter()
            .map(|peer| self.get_peer_network_id_for_peer(peer))
            .collect();
        Ok(self
            .network_client
            .to_bytes_by_protocol(peer_network_ids, message)?
            .into_iter()
            .map(|(peer_network_id, bytes)| (peer_network_id.peer_id(), bytes))
            .collect())
    }

    // TODO: we shouldn't need to expose this. Migrate the code to handle peer and network ids.
    fn get_peer_network_id_for_peer(&self, peer: PeerId) -> PeerNetworkId {
        PeerNetworkId::new(NetworkId::Validator, peer)
    }

    pub fn sort_peers_by_latency(&self, peers: &mut [PeerId]) {
        self.network_client
            .sort_peers_by_latency(NetworkId::Validator, peers)
    }
}
