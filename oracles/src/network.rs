// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network_interface::{OracleNetworkClient, RPC},
    OracleMessage,
};
use anyhow::bail;
use cedra_channels::{cedra_channel, message_queues::QueueStyle};
use cedra_config::network_id::NetworkId;
use cedra_infallible::RwLock;
use cedra_logger::warn;
use cedra_network::{
    application::interface::{NetworkClient, NetworkServiceEvents},
    protocols::network::{Event, RpcError},
    ProtocolId,
};
use cedra_reliable_broadcast::RBNetworkSender;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{
    stream::{select, select_all},
    SinkExt, Stream, StreamExt,
};
use futures_channel::oneshot;
use move_core_types::account_address::AccountAddress;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::timeout;

pub struct IncomingRpcRequest {
    pub msg: OracleMessage,
    pub sender: AccountAddress,
    pub response_sender: Box<dyn RpcResponseSender>,
}

/// Implements the actual networking support for all Oracle messaging.
#[derive(Clone)]
pub struct NetworkSender {
    author: AccountAddress,
    oracle_network_client: OracleNetworkClient<NetworkClient<OracleMessage>>,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    self_sender: cedra_channels::Sender<Event<OracleMessage>>,
}

impl NetworkSender {
    pub fn new(
        author: AccountAddress,
        oracle_network_client: OracleNetworkClient<NetworkClient<OracleMessage>>,
        self_sender: cedra_channels::Sender<Event<OracleMessage>>,
    ) -> Self {
        NetworkSender {
            author,
            oracle_network_client,
            self_sender,
        }
    }

    pub fn author(&self) -> AccountAddress {
        self.author
    }

    pub async fn send_rpc(
        &self,
        receiver: AccountAddress,
        msg: OracleMessage,
        timeout_duration: Duration,
    ) -> anyhow::Result<OracleMessage> {
        if receiver == self.author() {
            let (tx, rx) = oneshot::channel();
  let protocol = RPC.first().cloned().unwrap_or(ProtocolId::OracleRpcBcs);
  let self_msg = Event::RpcRequest(self.author, msg.clone(), RPC[0], tx);
            self.self_sender.clone().send(self_msg).await?;


            
               match timeout(timeout_duration, rx).await {
            Ok(Ok(Ok(bytes))) => {
                let bytes_vec = bytes.to_vec();
                let response_msg = tokio::task::spawn_blocking(move || {
                    protocol.from_bytes(&bytes_vec)
                }).await??;
                Ok(response_msg)
            }
            Ok(Ok(Err(rpc_error))) => {
                bail!("RPC error: {:?}", rpc_error);
            }
            Ok(Err(_)) => {
                bail!("Oneshot channel error");
            }
            Err(_) => {
                bail!("RPC timeout");
            }
        }        } else {
            Ok(self
                .oracle_network_client
                .send_rpc(receiver, msg, timeout_duration)
                .await?)
        }
    }
}

#[async_trait]
impl RBNetworkSender<OracleMessage> for NetworkSender {
    async fn send_rb_rpc_raw(
        &self,
        receiver: AccountAddress,
        raw_message: Bytes,
        timeout: Duration,
    ) -> anyhow::Result<OracleMessage> {
        Ok(self
            .oracle_network_client
            .send_rpc_raw(receiver, raw_message, timeout)
            .await?)
    }

    async fn send_rb_rpc(
        &self,
        receiver: AccountAddress,
        message: OracleMessage,
        timeout: Duration,
    ) -> anyhow::Result<OracleMessage> {
        self.send_rpc(receiver, message, timeout).await
    }

    fn to_bytes_by_protocol(
        &self,
        peers: Vec<AccountAddress>,
        message: OracleMessage,
    ) -> anyhow::Result<HashMap<AccountAddress, Bytes>> {
        self.oracle_network_client.to_bytes_by_protocol(peers, message)
    }

    fn sort_peers_by_latency(&self, peers: &mut [AccountAddress]) {
        self.oracle_network_client.sort_peers_by_latency(peers)
    }
}

pub struct NetworkReceivers {
    pub rpc_rx: cedra_channel::Receiver<AccountAddress, (AccountAddress, IncomingRpcRequest)>,
}

pub struct NetworkTask {
    all_events: Box<dyn Stream<Item = Event<OracleMessage>> + Send + Unpin>,
    rpc_tx: cedra_channel::Sender<AccountAddress, (AccountAddress, IncomingRpcRequest)>,
}

impl NetworkTask {
    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn new(
        network_service_events: NetworkServiceEvents<OracleMessage>,
        self_receiver: cedra_channels::Receiver<Event<OracleMessage>>,
    ) -> (NetworkTask, NetworkReceivers) {
        let (rpc_tx, rpc_rx) = cedra_channel::new(QueueStyle::FIFO, 10, None);

        let network_and_events = network_service_events.into_network_and_events();
        if (network_and_events.values().len() != 1)
            || !network_and_events.contains_key(&NetworkId::Validator)
        {
            panic!("The network has not been setup correctly for Oracle!");
        }

        // Collect all the network events into a single stream
        let network_events: Vec<_> = network_and_events.into_values().collect();
        let network_events = select_all(network_events).fuse();
        let all_events = Box::new(select(network_events, self_receiver));

        (NetworkTask { rpc_tx, all_events }, NetworkReceivers {
            rpc_rx,
        })
    }

    pub async fn start(mut self) {
        while let Some(message) = self.all_events.next().await {
            match message {
                Event::RpcRequest(peer_id, msg, protocol, response_sender) => {
                    let req = IncomingRpcRequest {
                        msg,
                        sender: peer_id,
                        response_sender: Box::new(RealRpcResponseSender {
                            inner: Some(response_sender),
                            protocol,
                        }),
                    };

                    if let Err(e) = self.rpc_tx.push(peer_id, (peer_id, req)) {
                        warn!(error = ?e, "cedra channel closed");
                    };
                },
                _ => {
                    // Ignored. Currently only RPC is used.
                },
            }
        }
    }
}

pub trait RpcResponseSender: Send + Sync {
    fn send(&mut self, response: anyhow::Result<OracleMessage>);
}

pub struct RealRpcResponseSender {
    pub inner: Option<oneshot::Sender<Result<Bytes, RpcError>>>,
    pub protocol: ProtocolId,
}

impl RealRpcResponseSender {
    pub fn new(raw_sender: oneshot::Sender<Result<Bytes, RpcError>>, protocol: ProtocolId) -> Self {
        Self {
            inner: Some(raw_sender),
            protocol,
        }
    }
}

impl RpcResponseSender for RealRpcResponseSender {
    fn send(&mut self, response: anyhow::Result<OracleMessage>) {
        let rpc_response = response
            .and_then(|oracle_msg| self.protocol.to_bytes(&oracle_msg).map(Bytes::from))
            .map_err(RpcError::ApplicationError);
        let _ = self.inner.take().unwrap().send(rpc_response); // May not succeed.
    }
}

pub struct DummyRpcResponseSender {
    pub rpc_response_collector: Arc<RwLock<Vec<anyhow::Result<OracleMessage>>>>,
}

impl DummyRpcResponseSender {
    pub fn new(rpc_response_collector: Arc<RwLock<Vec<anyhow::Result<OracleMessage>>>>) -> Self {
        Self {
            rpc_response_collector,
        }
    }
}

impl RpcResponseSender for DummyRpcResponseSender {
    fn send(&mut self, response: anyhow::Result<OracleMessage>) {
        self.rpc_response_collector.write().push(response);
    }
}
