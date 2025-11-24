use anyhow::{anyhow, Result};
use ethers::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;
use tracing::{info, warn};

// Same ABI file you already use in the deposit relayer crate
abigen!(
    EthBridge,
    "../Eth.Bridge.abi.json"
);

#[derive(Debug, Deserialize)]
struct CedraEvent {
    #[serde(rename = "type")]
    typ: String,
    data: Value,
}

#[derive(Debug, Deserialize)]
struct CedraTransaction {
    version: String,
    #[serde(default)]
    events: Vec<CedraEvent>,
}

// vector<u8> of 20 zeros => native ETH
const L1_ETH_BYTES_STRIPPED: &str = "0000000000000000000000000000000000000000";

/// Config that cedra-node will fill in from NodeConfig
#[derive(Clone, Debug)]
pub struct WithdrawRelayerConfig {
    pub cedra_rest_url: String,
    pub cedra_bridge_address: String, // hex string
    pub cedra_start_version: u64,
    pub eth_rpc_url: String,
    pub eth_bridge_address: Address,
    pub eth_chain_id: u64,
    pub poll_interval_ms: u64,
    pub eth_private_key: String,
}

/// Entry point used by cedra-node (and optionally by a bin)
pub async fn run_with_config(cfg: WithdrawRelayerConfig) -> Result<()> {
    let cedra_bridge_addr_lower = cfg.cedra_bridge_address.to_lowercase();
    let withdrawal_event_type =
        format!("{cedra_bridge_addr_lower}::bridge::Withdrawal").to_lowercase();

    // --- Ethereum side: wallet + contract ---
    let provider = Provider::<Http>::try_from(cfg.eth_rpc_url.clone())?;
    let provider = SignerMiddleware::new(
        provider,
        LocalWallet::from_str(&cfg.eth_private_key)?.with_chain_id(cfg.eth_chain_id),
    );
    let provider = std::sync::Arc::new(provider);

    let bridge = EthBridge::new(cfg.eth_bridge_address, provider.clone());
    let relayer_eoa = provider.address();

    info!("Starting Cedra→Eth withdrawal relayer...");
    info!("Cedra REST: {}", cfg.cedra_rest_url);
    info!("Cedra bridge addr: {}", cfg.cedra_bridge_address);
    info!("Eth bridge addr: {:?}", cfg.eth_bridge_address);
    info!("Relayer EOA: {:?}", relayer_eoa);
    info!("Starting from Cedra tx version: {}", cfg.cedra_start_version);

    let mut next_version = cfg.cedra_start_version;

    loop {
        if let Err(e) = poll_once(
            &cfg.cedra_rest_url,
            &withdrawal_event_type,
            &bridge,
            &mut next_version,
        )
        .await
        {
            warn!("poll_once error: {e:?}");
        }

        sleep(Duration::from_millis(cfg.poll_interval_ms)).await;
    }
}

// One polling step
async fn poll_once(
    cedra_rest_url: &str,
    withdrawal_event_type: &str,
    bridge: &EthBridge<SignerMiddleware<Provider<Http>, LocalWallet>>,
    next_version: &mut u64,
) -> Result<()> {
    let url = format!(
        "{}/v1/transactions?start={}&limit=100",
        cedra_rest_url, next_version
    );

    let resp = reqwest::get(&url).await?;
    let status = resp.status();

    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Cedra /v1/transactions error {}: {}",
            status,
            text
        ));
    }

    let txs: Vec<CedraTransaction> = resp.json().await?;
    if txs.is_empty() {
        return Ok(());
    }

    for tx in txs {
        let tx_version: u64 = tx.version.parse()?;

        if tx_version >= *next_version {
            *next_version = tx_version + 1;
        }

        if tx.events.is_empty() {
            continue;
        }

        for ev in tx.events {
            if ev.typ.to_lowercase() != withdrawal_event_type {
                continue;
            }

            handle_withdrawal_event(tx_version, ev, bridge).await?;
        }
    }

    Ok(())
}

async fn handle_withdrawal_event(
    version: u64,
    ev: CedraEvent,
    bridge: &EthBridge<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<()> {
    let d = ev.data;

    let l1_token_hex = d
        .get("l1_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Withdrawal.l1_token missing"))?;
    let eth_recipient_hex = d
        .get("eth_recipient")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Withdrawal.eth_recipient missing"))?;
    let amount_str = d
        .get("amount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Withdrawal.amount missing"))?;
    let nonce_str = d
        .get("nonce")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Withdrawal.nonce missing"))?;

    let token_addr = l1_token_bytes_to_eth_token(l1_token_hex)?;
    let eth_recipient = normalize_eth_address(eth_recipient_hex)?;

    let amount = U256::from_dec_str(amount_str)?;
    let nonce_u64: u64 = nonce_str.parse()?;
    let nonce_u256 = U256::from(nonce_u64);

    info!(
        "[Cedra v{}] Withdrawal event: token={:?}, to={:?}, amount={}, nonce={}",
        version, token_addr, eth_recipient, amount, nonce_u64
    );

    let mut call = bridge.withdraw(eth_recipient, token_addr, amount, nonce_u256);
    let gas_estimate = call.estimate_gas().await?;
    let gas_limit = gas_estimate * 120u32 / 100u32;
    let call = call.gas(gas_limit);

    let pending_tx = call.send().await?;
    info!("→ Sent withdraw tx to Ethereum: {:?}", pending_tx.tx_hash());

    match pending_tx.await {
        Ok(Some(receipt)) => {
            info!(
                "✓ withdraw confirmed, status: {:?}, gasUsed: {:?}",
                receipt.status, receipt.gas_used
            );
        }
        Ok(None) => {
            warn!("withdraw tx pending or dropped (no receipt yet)");
        }
        Err(e) => {
            warn!("error waiting for withdraw receipt: {e:?}");
        }
    }

    Ok(())
}

// --- helpers ---

fn normalize_eth_address(s: &str) -> Result<Address> {
    let mut v = s.trim().to_string();
    if !v.starts_with("0x") {
        v = format!("0x{}", v);
    }
    let without_0x = &v[2..];
    if without_0x.len() != 40 {
        return Err(anyhow!("expected 20-byte address hex, got: {}", s));
    }
    Ok(Address::from_str(&v)?)
}

fn l1_token_bytes_to_eth_token(s: &str) -> Result<Address> {
    let mut v = s.trim().to_string();
    if !v.starts_with("0x") {
        v = format!("0x{}", v);
    }
    let without_0x = &v[2..];

    let stripped = if without_0x.len() < 40 {
        format!("{:0>40}", without_0x)
    } else {
        without_0x.to_string()
    };

    if stripped.eq_ignore_ascii_case(L1_ETH_BYTES_STRIPPED) {
        return Ok(Address::zero());
    }

    let full = format!("0x{}", stripped);
    Ok(Address::from_str(&full)?)
}