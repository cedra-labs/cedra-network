use anyhow::*;
use ethers::prelude::*;
use tokio_stream::StreamExt;
use tracing::{info, warn};
use std::sync::Arc;
use std::result::Result::Ok;

use aptos_sdk::{
    bcs,
    crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt},
    rest_client::Client as AptosClient,
    transaction_builder::TransactionFactory,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        LocalAccount,
        transaction::TransactionPayload,
    },
};
use url::Url;

mod build_execute_deposit;
use build_execute_deposit::build_execute_deposit_entry;

abigen!(
    EthBridge,
    "../Eth.Bridge.abi.json"
);

/// Config for eth→cedra relayer
#[derive(Clone, Debug)]
pub struct EthToCedraRelayerConfig {
    pub eth_rpc_url: String,
    pub eth_bridge_address: Address,
    pub eth_start_block: Option<u64>,

    pub cedra_rest_url: String,
    pub cedra_chain_id: u8,

    pub cedra_private_key: String,        // hex or "ed25519-priv-..."
    pub cedra_account_address: AccountAddress,
    pub cedra_bridge_module_address: AccountAddress, // address that has module `bridge::execute_deposit`

    pub cedra_gas_unit_price: u64,
    pub cedra_max_gas: u64,
}

/// Entry point used by cedra-node
pub async fn run_with_config(cfg: EthToCedraRelayerConfig) -> Result<()> {
    // =============== Ethereum side ===============
    let provider = Provider::<Http>::try_from(cfg.eth_rpc_url.clone())?;
    let provider = Arc::new(provider);
    let bridge = EthBridge::new(cfg.eth_bridge_address, provider.clone());

    let from_block = cfg.eth_start_block
        .map(|h| BlockNumber::Number(U64::from(h)))
        .unwrap_or(BlockNumber::Latest);

    let filter = bridge.deposit_filter().from_block(from_block);
    let mut stream = filter.stream().await?.with_meta();

    info!("Listening for Deposit events from {:?}…", cfg.eth_bridge_address);

    // =============== Cedra side ===============
    let cedra_url = Url::parse(&cfg.cedra_rest_url)?;
    let chain_id = ChainId::new(cfg.cedra_chain_id);
    let client = AptosClient::new(cedra_url);

    let mut priv_hex = cfg.cedra_private_key.clone();
    if let Some(stripped) = priv_hex.strip_prefix("ed25519-priv-") {
        priv_hex = stripped.to_string();
    }
    let priv_key = Ed25519PrivateKey::from_encoded_string(&priv_hex)?;
    let mut account = LocalAccount::new(cfg.cedra_account_address, priv_key, 0);

    let gas_unit_price = cfg.cedra_gas_unit_price;
    let max_gas = cfg.cedra_max_gas;

    let tf = TransactionFactory::new(chain_id)
        .with_gas_unit_price(gas_unit_price)
        .with_max_gas_amount(max_gas);

    while let Some(item) = stream.next().await {
        match item {
            Ok((event, meta)) => {
                // event: Deposit(token, from, cedraTo, amount, nonce)
                let tx_hash: Vec<u8> = meta.transaction_hash.0.to_vec();

                let l1_token_20 = if event.token == Address::zero() {
                    vec![0u8; 20]
                } else {
                    event.token.0.to_vec()
                };

                let cedra_to_32: [u8; 32] = event.cedra_to.into();
                let to_l2 = match aptos_sdk::types::account_address::AccountAddress::from_bytes(cedra_to_32) {
                    Ok(addr) => addr,
                    Err(_) => {
                        warn!("Deposit.cedra_to is not a valid 32-byte Cedra address");
                        continue;
                    }
                };

                let amount_u64: u64 = match event.amount.try_into() {
                    Ok(v) => v,
                    Err(_) => {
                        warn!("amount too large for u64, skipping");
                        continue;
                    }
                };
                let nonce_u64 = event.nonce.as_u64();

                info!(
                    "↳ deposit token={:?} cedra_to={} amount={} nonce={} eth_tx={:?}",
                    event.token, to_l2, amount_u64, nonce_u64, meta.transaction_hash
                );

                // Build EntryFunction using your helper
                let entry = build_execute_deposit_entry(
                    cfg.cedra_bridge_module_address,
                    l1_token_20,
                    to_l2,
                    amount_u64,
                    nonce_u64,
                    tx_hash,
                );

                let payload = TransactionPayload::EntryFunction(entry);

                // Refresh seq
                let acct = client.get_account(account.address()).await?.into_inner();
                account.set_sequence_number(acct.sequence_number);

                let txn = account.sign_with_transaction_builder(tf.payload(payload));
                let resp = client.submit(&txn).await?;
                let pending = resp.into_inner();
                let committed = client.wait_for_transaction(&pending).await?;
                let tx = committed.into_inner();

                match tx {
                    aptos_sdk::rest_client::Transaction::UserTransaction(utx) => {
                        if utx.info.success {
                            info!("✅ Cedra execute_deposit OK: {}", utx.info.hash);
                        } else {
                            warn!("❌ Cedra tx failed: {} -- {:?}", utx.info.vm_status, utx);
                        }
                    }
                    other => warn!("unexpected transaction kind: {:?}", other),
                }
            }
            Err(e) => {
                warn!("event stream error: {e:?}");
            }
        }
    }

    Ok(())
}