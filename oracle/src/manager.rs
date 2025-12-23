use crate::config::ORACLE_AUTH_KEY_FILE;
use crate::whitelist::Whitelist;

use anyhow::{Context, Result};
use cedra_event_notifications::EventNotificationListener;
use cedra_types::{
    chain_id::ChainId,
    oracle::PriceInfo,
    validator_txn::{Topic, ValidatorTransaction},
};
use cedra_validator_transaction_pool::{TxnGuard, VTxnPoolState};
use futures::StreamExt;
use std::{collections::HashMap, fs, sync::Arc};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use cedra_logger::{debug, error};
use tonic::{
    metadata::{Ascii, MetadataValue},
    Request,
};

pub mod pricefeed {
    tonic::include_proto!("stream");
}

use pricefeed::{stream_service_client::StreamServiceClient, PriceRequest};

pub struct OraclePriceManager {
    // In-memory whitelist that updated on Event emitted by add or remove asset to whitelist in move
    whitelist: Arc<Whitelist>,
    vtxn_pool: VTxnPoolState,
    oracles_updated_events: EventNotificationListener,
    // All whitelist assets streams
    active_streams: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    active_guards: Arc<Mutex<Vec<TxnGuard>>>,
    // auth key for grpc oracle connection
    auth_key: String,
    chain_id: ChainId,
}

impl OraclePriceManager {
    pub fn new(
        whitelist: Arc<Whitelist>,
        vtxn_pool: VTxnPoolState,
        oracles_updated_events: EventNotificationListener,
        chain_id: ChainId,
    ) -> Self {
        let auth_key = fs::read_to_string(ORACLE_AUTH_KEY_FILE)
            .map_err(|e| anyhow::anyhow!("Failed to read auth key: {}", e))
            .unwrap()
            .trim()
            .to_string();

        Self {
            whitelist,
            vtxn_pool,
            oracles_updated_events,
            active_streams: Arc::new(Mutex::new(HashMap::new())),
            active_guards: Arc::new(Mutex::new(Vec::new())),
            auth_key,
            chain_id,
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

    pub async fn run(
        &mut self,
        price_rx: mpsc::Receiver<PriceInfo>,
        price_tx: mpsc::Sender<PriceInfo>,
    ) -> Result<()> {
        // Initial update of whitelist on oracle runtime start
        self.whitelist.update_whitelist().await;

        self.ensure_whitelist_streams(price_tx.clone()).await?;

        self.spawn_price_submitter(price_rx).await;
        self.listen_for_events(price_tx.clone()).await?;

        Ok(())
    }

    async fn ensure_whitelist_streams(&self, price_tx: mpsc::Sender<PriceInfo>) -> Result<()> {
        let whitelist = self.whitelist.get_whitelist();
        let whitelist_addresses: Vec<String> = whitelist.iter().map(|a| a.move_type_string()).collect();
        let mut active_streams = self.active_streams.lock().await;

         let to_remove: Vec<String> = active_streams
        .keys()
        .filter(|addr| !whitelist_addresses.contains(addr))
        .cloned()
        .collect();

        for fa_address in to_remove {
            debug!("🛑 Stopping stream for removed asset: {:?}", fa_address);
            if let Some(handle) = active_streams.remove(&fa_address) {
                handle.abort(); // stop the stream task
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

                let handle = tokio::spawn(async move {
                    if let Err(e) =
                        Self::price_stream_task(for_task, auth_key, price_tx, server_address).await
                    {
                        error!("❌ Stream management failed for {:?}: {}", &fa_address, e);
                    }
                });

                active_streams.insert(for_map, handle);
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
        server_address: String,
    ) -> Result<()> {
        let price_tx = price_tx.clone();

        let header_value = MetadataValue::try_from(auth)
            .map_err(|e| anyhow::anyhow!("Failed to create metadata value: {}", e))?;

        loop {
            match Self::open_price_stream(
                &fa_address.clone(),
                header_value.clone(),
                price_tx.clone(),
                server_address.clone(),
            )
            .await
            {
                Ok(_) => {
                    debug!("⚠️  Stream for {} closed, reconnecting...", fa_address);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                },
                Err(e) => {
                    error!("❌ Stream error for {}: {}, reconnecting...", fa_address, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                },
            }
        }
    }

    async fn open_price_stream(
        fa_address: &str,
        auth: MetadataValue<Ascii>,
        price_tx: mpsc::Sender<PriceInfo>,
        server_address: String,
    ) -> Result<()> {
        let channel = tonic::transport::Channel::from_shared(server_address)
            .map_err(|e| anyhow::anyhow!("Invalid server address: {}", e))?
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

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

        while let Some(price_update) = stream.next().await {
            match price_update {
                Ok(token) => {
                    if price_tx
                        .send(PriceInfo {
                            fa_address: token.fa_address.clone(),
                            price: token.price,
                            decimals: token.decimals as u8,
                        })
                        .await
                        .is_err()
                    {
                        return Err(anyhow::anyhow!("Batch channel closed"));
                    }
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("Stream error for {:?}: {}", fa_address, e).into());
                },
            }
        }

        Ok(())
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
