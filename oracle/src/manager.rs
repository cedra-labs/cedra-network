use crate::whitelist::Whitelist;

use anyhow::{Context, Result};
use cedra_event_notifications::EventNotificationListener;
use cedra_logger::{debug, error};
use cedra_types::{
    chain_id::ChainId,
    oracle::PriceInfo,
    validator_txn::{Topic, ValidatorTransaction},
};
use cedra_validator_transaction_pool::{TxnGuard, VTxnPoolState};
use futures::StreamExt;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tonic::transport::{Channel, Endpoint};
use tonic::{
    metadata::{Ascii, MetadataValue},
    Request,
};

pub mod pricefeed {
    tonic::include_proto!("stream");
}

use pricefeed::{stream_service_client::StreamServiceClient, PriceRequest};

#[derive(Clone)]
pub struct OraclePriceManagerHandle {
    grpc_pool: Arc<Mutex<Option<Channel>>>,
    server_address: String,
}

pub struct OraclePriceManager {
    // In-memory whitelist that updated on Event emitted by add or remove asset to whitelist in move
    whitelist: Arc<Whitelist>,
    vtxn_pool: VTxnPoolState,
    oracles_updated_events: EventNotificationListener,
    // All whitelist assets streams
    active_streams: Arc<Mutex<HashMap<String, (JoinHandle<()>, oneshot::Sender<()>)>>>,
    grpc_pool: Arc<Mutex<Option<Channel>>>,
    active_guards: Arc<Mutex<Vec<TxnGuard>>>,
    // auth key for grpc oracle connection
    auth_key: String,
    chain_id: ChainId,
}

impl OraclePriceManagerHandle {
    async fn get_or_create_channel(&self) -> Result<Channel> {
        let mut pool = self.grpc_pool.lock().await;
        if let Some(channel) = pool.as_ref() {
            return Ok(channel.clone());
        }

        let endpoint = Endpoint::from_shared(self.server_address.clone())
            .map_err(|e| anyhow::anyhow!("Invalid server address: {}", e))?
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(20))
            .keep_alive_while_idle(true)
            .concurrency_limit(100);

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

        *pool = Some(channel.clone());
        Ok(channel)
    }
}

impl OraclePriceManager {
    pub fn new(
        auth_key: String,
        whitelist: Arc<Whitelist>,
        vtxn_pool: VTxnPoolState,
        oracles_updated_events: EventNotificationListener,
        chain_id: ChainId,
    ) -> Self {
        Self {
            whitelist,
            vtxn_pool,
            oracles_updated_events,
            active_streams: Arc::new(Mutex::new(HashMap::new())),
            grpc_pool: Arc::new(Mutex::new(None)),
            active_guards: Arc::new(Mutex::new(Vec::new())),
            auth_key,
            chain_id,
        }
    }

    pub fn clone_for_task(&self) -> OraclePriceManagerHandle {
        OraclePriceManagerHandle {
            grpc_pool: Arc::clone(&self.grpc_pool),
            server_address: self.server_address(),
        }
    }

    fn server_address(&self) -> String {
        //todo: add mainnet and change testnet rpc urls
        if self.chain_id.is_mainnet() {
            "work_in_progress".to_string()
        } else if self.chain_id.is_testnet() {
            "http://d-rpc-oracle.cedra.dev:40042".to_string()
        } else {
            "http://d-rpc-oracle.cedra.dev:40042".to_string()
        }
    }

    pub async fn cleanup_all_streams(&mut self) -> Result<()> {
        debug!("🧹 Cleaning up all streams and resources...");

        let mut streams = self.active_streams.lock().await;
        let mut tasks = Vec::new();

        for (addr, (handle, shutdown_tx)) in streams.drain() {
            debug!("Stopping stream for: {}", addr);
            let _ = shutdown_tx.send(());
            handle.abort();
            tasks.push(addr);
        }

        drop(streams);

        let mut guards = self.active_guards.lock().await;
        guards.clear();

        tokio::time::sleep(Duration::from_millis(500)).await;

        debug!("✅ Cleaned up {} streams", tasks.len());
        Ok(())
    }

    pub async fn run(
        &mut self,
        price_rx: mpsc::Receiver<PriceInfo>,
        price_tx: mpsc::Sender<PriceInfo>,
    ) -> Result<()> {
        let _ = self.cleanup_all_streams().await;
        // Initial update of whitelist on oracle runtime start
        self.whitelist.update_whitelist().await;

        self.ensure_whitelist_streams(price_tx.clone()).await?;

        self.spawn_price_submitter(price_rx).await;
        self.listen_for_events(price_tx.clone()).await?;

        Ok(())
    }

    async fn ensure_whitelist_streams(&self, price_tx: mpsc::Sender<PriceInfo>) -> Result<()> {
        let whitelist = self.whitelist.get_whitelist();
        let whitelist_addresses: Vec<String> =
            whitelist.iter().map(|a| a.move_type_string()).collect();
        const MAX_STREAMS: usize = 300;

        let mut active_streams = self.active_streams.lock().await;

        // Check for max streams before adding new ones
        if active_streams.len() >= MAX_STREAMS {
            error!(
                "❌ Cannot start new streams: maximum number of active streams ({}) reached!",
                MAX_STREAMS
            );
            // Skip starting new streams
            return Ok(());
        }
        let to_remove: Vec<String> = active_streams
            .keys()
            .filter(|addr| !whitelist_addresses.contains(addr))
            .cloned()
            .collect();

        for fa_address in to_remove {
            debug!("🛑 Stopping stream for removed asset: {:?}", fa_address);
            if let Some((join_handle, shutdown_tx)) = active_streams.remove(&fa_address) {
                let _ = shutdown_tx.send(());
                tokio::time::sleep(Duration::from_millis(100)).await;
                join_handle.abort();
            }

            let txn = ValidatorTransaction::RemovePrice(fa_address.clone());
            let topic = Topic::from("oracle_prices");
            let txn_guard = self.vtxn_pool.put(topic, Arc::new(txn), None);
            let mut guards = self.active_guards.lock().await;
            guards.push(txn_guard);
        }

        for asset in whitelist {
            let fa_address = asset.move_type_string();
            let for_task = fa_address.clone();
            let for_map = fa_address.clone();

            if !active_streams.contains_key(&fa_address) {
                debug!("➕ Starting stream for new asset: {:?}", fa_address);

                let auth_key = self.auth_key.clone();
                let server_address = self.server_address();
                let price_tx = price_tx.clone();

                let manager_handle = OraclePriceManagerHandle {
                    grpc_pool: Arc::clone(&self.grpc_pool),
                    server_address: server_address.clone(),
                };

                let (shutdown_tx, shutdown_rx) = oneshot::channel();

                let handle = tokio::spawn(async move {
                    if let Err(e) = Self::price_stream_task(
                        for_task,
                        auth_key,
                        price_tx,
                        &manager_handle,
                        shutdown_rx,
                    )
                    .await
                    {
                        error!("❌ Stream management failed for {:?}: {}", &fa_address, e);
                    }
                });

                active_streams.insert(for_map, (handle, shutdown_tx));
            }
        }

        println!(
            "[OraclePriceManager] ✅ Active streams: {}",
            active_streams.len()
        );

        Ok(())
    }

    async fn price_stream_task(
        fa_address: String,
        auth: String,
        price_tx: mpsc::Sender<PriceInfo>,
        handle: &OraclePriceManagerHandle,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<()> {
        let price_tx = price_tx.clone();

        let header_value = MetadataValue::try_from(auth)
            .map_err(|e| anyhow::anyhow!("Failed to create metadata value: {}", e))?;

        let mut reconnect_backoff = 1;
        const MAX_BACKOFF: u64 = 30;

        loop {
            if shutdown_rx.try_recv().is_ok() {
                debug!("Shutdown signal received for stream: {}", fa_address);
                return Ok(());
            }

            match Self::open_price_stream(
                &fa_address,
                header_value.clone(),
                price_tx.clone(),
                handle,
                &mut shutdown_rx,
            )
            .await
            {
                Ok(_) => {
                    debug!("Stream for {} closed normally, reconnecting...", fa_address);
                    reconnect_backoff = 1;
                },
                Err(e) => {
                    error!(
                        "Stream error for {}: {}, reconnecting in {}s...",
                        fa_address, e, reconnect_backoff
                    );

                    // Check for shutdown before sleeping
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(reconnect_backoff)) => {},
                        _ = &mut shutdown_rx => return Ok(()),
                    }

                    // Exponential backoff with cap
                    reconnect_backoff = std::cmp::min(reconnect_backoff * 2, MAX_BACKOFF);
                    continue;
                },
            }

            // Short delay before reconnection
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    async fn open_price_stream(
        fa_address: &str,
        auth: MetadataValue<Ascii>,
        price_tx: mpsc::Sender<PriceInfo>,
        handle: &OraclePriceManagerHandle,
        shutdown_rx: &mut oneshot::Receiver<()>,
    ) -> Result<()> {
        let channel = handle.get_or_create_channel().await?;
        let mut client = StreamServiceClient::new(channel);

        let mut request = Request::new(PriceRequest {
            fa_address: fa_address.to_string(),
        });

        request.metadata_mut().insert("authorization", auth);

        let response = client
            .white_list_price_stream(request)
            .await
            .with_context(|| format!("Failed to start stream for {:?}", fa_address))?;

        let mut stream = response.into_inner();

        loop {
            tokio::select! {
                // Check for shutdown
                _ = &mut *shutdown_rx => {
                    debug!("Shutdown during stream for {}", fa_address);
                    return Ok(());
                }
                // Receive price updates
                price_update = stream.next() => match price_update {
                    Some(Ok(token)) => {
                        if price_tx
                            .send(PriceInfo {
                                fa_address: token.fa_address.clone(),
                                price: token.price,
                                decimals: token.decimals as u8,
                                timestamp: token.timestamp / 1000 as u64,
                            })
                            .await
                            .is_err()
                        {
                            return Err(anyhow::anyhow!("Batch channel closed"));
                        }
                    },
                    Some(Err(e)) => {
                        return Err(anyhow::anyhow!("Stream error for {:?}: {}", fa_address, e));
                    },
                    None => {
                        debug!("Stream ended normally for {}", fa_address);
                        return Ok(());
                    },
                },
            }
        }
    }

    async fn spawn_price_submitter(&mut self, mut price_rx: mpsc::Receiver<PriceInfo>) {
        let vtxn_pool = self.vtxn_pool.clone();
        let active_guards = self.active_guards.clone();

        tokio::spawn(async move {
            let mut buffer: HashMap<String, PriceInfo> = HashMap::new();
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(1000));

            loop {
                tokio::select! {
                    Some(price) = price_rx.recv() => {
                         buffer.insert(price.fa_address.clone(), price.clone());
                    }
                    _ = ticker.tick() => {
                        if !buffer.is_empty() {

                            let batch: Vec<PriceInfo> = buffer.values().cloned().collect();
                            buffer.clear();

                            let txn = ValidatorTransaction::AddPrice(batch.clone());
                            let topic = Topic::from("oracle_prices");

                            let txn_guard = vtxn_pool.put(topic, Arc::new(txn), None);

                            let mut guards = active_guards.lock().await;

                            guards.push(txn_guard);

                            if guards.len() > 100 {
                                guards.drain(0..50);
                            }

                        }
                    }
                }
            }
        });
    }

    async fn listen_for_events(&mut self, price_tx: mpsc::Sender<PriceInfo>) -> Result<()> {
        loop {
            tokio::select! {
                // Handle events from the event stream
                _ = self.oracles_updated_events.select_next_some() => {
                    self.whitelist.update_whitelist().await;
                    self.ensure_whitelist_streams(price_tx.clone()).await?;
                }
            }
        }
    }
}
