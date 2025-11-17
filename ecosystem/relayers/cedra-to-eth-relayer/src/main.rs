use anyhow::{anyhow, Result};
use ethers::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;
use tracing::{info, warn};
use std::env;

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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::from_filename("ecosystem/relayers/cedra-to-eth-relayer/.env").ok();

    if let Ok(val) = env::var("CEDRA_START_VERSION") {
        info!("RAW CEDRA_START_VERSION from env: {:?}", val);
    } else {
        info!("CEDRA_START_VERSION not present in env");
    }

    tracing_subscriber::fmt().init();

    // --- ENV ---
    let cedra_rest_url =
        std::env::var("CEDRA_REST_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let cedra_bridge_addr_raw = std::env::var("CEDRA_BRIDGE_ADDRESS")
        .map_err(|_| anyhow!("CEDRA_BRIDGE_ADDRESS env var missing"))?;
    let cedra_bridge_addr = cedra_bridge_addr_raw.to_lowercase();

    let eth_rpc_url = std::env::var("ETH_RPC_URL")?;
    let eth_bridge_address: Address = env::var("ETH_BRIDGE_ADDRESS")?.parse()?;
    let relayer_pk = std::env::var("ETH_RELAYER_PRIVATE_KEY")?;
    let poll_interval_ms: u64 = std::env::var("POLL_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    // Start version to scan from
    let mut next_version: u64 = match env::var("CEDRA_START_VERSION") {
        Ok(s) => {
            info!("CEDRA_START_VERSION from env: {}", s);
            match s.parse::<u64>() {
                Ok(v) => v,
                Err(e) => {
                    warn!("Failed to parse CEDRA_START_VERSION='{}': {e}, defaulting to 0", s);
                    0
                }
            }
        }
        Err(_) => {
            info!("CEDRA_START_VERSION not set, defaulting to 0");
            0
        }
    };

    // Event type string exactly as indexer shows it:
    // e.g. "0xd3c5...::bridge::Withdrawal"
    let withdrawal_event_type =
        format!("{cedra_bridge_addr}::bridge::Withdrawal").to_lowercase();

    // --- Ethereum side: wallet + contract ---
    let provider = Provider::<Http>::try_from(eth_rpc_url)?;
    let provider = SignerMiddleware::new(
        provider,
        LocalWallet::from_str(&relayer_pk)?.with_chain_id(11155111u64), // adjust chain id if needed
    );
    let provider = std::sync::Arc::new(provider);

    let bridge = EthBridge::new(eth_bridge_address, provider.clone());
    let relayer_eoa = provider.address();

    info!("Starting Cedra→Eth withdrawal relayer...");
    info!("Cedra REST: {cedra_rest_url}");
    info!("Cedra bridge addr: {cedra_bridge_addr_raw}");
    info!("Eth bridge addr: {eth_bridge_address:?}");
    info!("Relayer EOA: {relayer_eoa:?}");
    info!("Starting from Cedra tx version: {next_version}");

    loop {
        if let Err(e) = poll_once(
            &cedra_rest_url,
            &withdrawal_event_type,
            &bridge,
            &mut next_version,
        )
        .await
        {
            warn!("poll_once error: {e:?}");
        }

        sleep(Duration::from_millis(poll_interval_ms)).await;
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
        // resp is moved here, which is fine because we no longer use it afterwards
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
    // Withdrawal event:
    // struct Withdrawal {
    //   l1_token: vector<u8>,
    //   from: address,
    //   eth_recipient: vector<u8>,
    //   amount: u64,
    //   nonce: u64,
    // }

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

    // L1 token (20 bytes)
    let token_addr = l1_token_bytes_to_eth_token(l1_token_hex)?;

    // ETH recipient (20 bytes address)
    let eth_recipient = normalize_eth_address(eth_recipient_hex)?;

    // Amount and nonce
    let amount = U256::from_dec_str(amount_str)?;
    let nonce_u64: u64 = nonce_str.parse()?;
    let nonce_u256 = U256::from(nonce_u64);

    info!(
        "[Cedra v{}] Withdrawal event: token={:?}, to={:?}, amount={}, nonce={}",
        version, token_addr, eth_recipient, amount, nonce_u64
    );

    // Call Ethereum Bridge.withdraw(to, token, amount, nonce)
    // NOTE: relayer must have WITHDRAWER_ROLE on the Bridge contract.
    let gas_limit = U256::from(100_000u64); // plenty of headroom

    // Call Ethereum Bridge.withdraw(to, token, amount, nonce)
    let mut call = bridge.withdraw(eth_recipient, token_addr, amount, nonce_u256);

    // Estimate gas instead of hard-coding
    let gas_estimate = call.estimate_gas().await?;
    let gas_limit = gas_estimate * 120 / 100; // +20% safety margin

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

/// Convert l1_token bytes (20 bytes) to ETH token address:
/// - 20 zero bytes => native ETH => Address::zero()
/// - anything else => treat as ERC20 token address on Ethereum
fn l1_token_bytes_to_eth_token(s: &str) -> Result<Address> {
    let mut v = s.trim().to_string();
    if !v.starts_with("0x") {
        v = format!("0x{}", v);
    }
    let without_0x = &v[2..];

    // pad to 40 chars
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