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

// vector<u8> of 20 zeros => native ETH
const L1_ETH_BYTES_STRIPPED: &str = "0000000000000000000000000000000000000000";

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

/// Config that cedra-node will fill in from NodeConfig
#[derive(Clone, Debug)]
pub struct WithdrawRelayerConfig {
    // Cedra side
    pub cedra_rest_url: String,     // e.g. http://localhost:8080
    pub cedra_bridge_address: String, // 0x... (Move module address)
    pub cedra_start_version: u64,

    /// Cedra chain id as seen from Ethereum bridge (same one you used on ETH side).
    pub cedra_chain_id_on_eth: u16,

    // Ethereum side
    pub eth_rpc_url: String,
    pub eth_bridge_address: Address,
    pub eth_chain_id: u64,
    pub poll_interval_ms: u64,
    pub eth_private_key: String,
}

/// Entry point used by cedra-node (and optionally by a bin)
pub async fn run_with_config(cfg: WithdrawRelayerConfig) -> Result<()> {
    let cedra_bridge_addr_lower = cfg.cedra_bridge_address.to_lowercase();
    // We now listen to Deposit events (Cedra -> ETH direction)
    let deposit_event_type =
        format!("{cedra_bridge_addr_lower}::bridge::Deposit").to_lowercase();

    // --- Ethereum side: wallet + contract ---
    let provider = Provider::<Http>::try_from(cfg.eth_rpc_url.clone())?;
    let provider = SignerMiddleware::new(
        provider,
        LocalWallet::from_str(&cfg.eth_private_key)?.with_chain_id(cfg.eth_chain_id),
    );
    let provider = std::sync::Arc::new(provider);

    let bridge = EthBridge::new(cfg.eth_bridge_address, provider.clone());
    let relayer_eoa = provider.address();

    info!("Starting Cedra→Eth deposit relayer...");
    info!("Cedra REST: {}", cfg.cedra_rest_url);
    info!("Cedra bridge addr: {}", cfg.cedra_bridge_address);
    info!("Eth bridge addr: {:?}", cfg.eth_bridge_address);
    info!("Relayer EOA: {:?}", relayer_eoa);
    info!("Starting from Cedra tx version: {}", cfg.cedra_start_version);

    let mut next_version = cfg.cedra_start_version;

    loop {
        if let Err(e) = poll_once(
            &cfg,
            &deposit_event_type,
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
    cfg: &WithdrawRelayerConfig,
    deposit_event_type: &str,
    bridge: &EthBridge<SignerMiddleware<Provider<Http>, LocalWallet>>,
    next_version: &mut u64,
) -> Result<()> {
    let url = format!(
        "{}/v1/transactions?start={}&limit=100",
        cfg.cedra_rest_url, next_version
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
            if ev.typ.to_lowercase() != deposit_event_type {
                continue;
            }

            handle_deposit_event(tx_version, ev, cfg, bridge).await?;
        }
    }

    Ok(())
}

async fn handle_deposit_event(
    version: u64,
    ev: CedraEvent,
    cfg: &WithdrawRelayerConfig,
    bridge: &EthBridge<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<()> {
    let d = ev.data;

    // Deposit fields from bridge.move:
    // struct Deposit {
    //   asset: address,
    //   from: address,
    //   remote_recipient: vector<u8>, // 20 bytes ETH address
    //   amount: u64,
    //   nonce: u64
    // }

    let asset_meta_hex = d
        .get("asset")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Deposit.asset missing"))?;

    let remote_recipient_hex = d
        .get("remote_recipient")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Deposit.remote_recipient missing"))?;

    let amount_str = d
        .get("amount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Deposit.amount missing"))?;
    let nonce_str = d
        .get("nonce")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Deposit.nonce missing"))?;

    let eth_recipient = bytes_vec_to_eth_address(remote_recipient_hex)?;
    let amount = U256::from_dec_str(amount_str)?;
    let nonce_u64: u64 = nonce_str.parse()?;
    let nonce_u256 = U256::from(nonce_u64);

    info!(
        "[Cedra v{}] Deposit event: asset_meta={}, eth_to={:?}, amount={}, nonce={}",
        version, asset_meta_hex, eth_recipient, amount, nonce_u64
    );

    // 1) Ask Cedra if this `asset_meta` is wrapped or native
    let is_wrapped = cedra_is_wrapped_asset(
        &cfg.cedra_rest_url,
        &cfg.cedra_bridge_address,
        asset_meta_hex,
    )
    .await?;

    if is_wrapped {
        // ---------------------------
        // Wrapped path:
        //   Cedra-wrapped token that represents some L1 ETH/ ERC20.
        //   We must get the origin L1 token and call `withdraw(to, token, amount, nonce)`.
        // ---------------------------

        let origin_token_hex = cedra_origin_token_of_wrapped(
            &cfg.cedra_rest_url,
            &cfg.cedra_bridge_address,
            asset_meta_hex,
        )
        .await?;

        let token_addr = l1_token_bytes_to_eth_token(&origin_token_hex)?;

        info!(
            "→ Classified as WRAPPED (ETH-origin). L1 token = {:?}",
            token_addr
        );

        let mut call = bridge.withdraw(eth_recipient, token_addr, amount, nonce_u256);
        let gas_estimate = call.estimate_gas().await?;
        let gas_limit = gas_estimate * 120u32 / 100u32;
        let call = call.gas(gas_limit);

        let pending_tx = call.send().await?;
        info!(
            "→ Sent withdraw() tx to Ethereum: {:?}",
            pending_tx.tx_hash()
        );

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
    } else {
        // ---------------------------
        // Native path:
        //   Cedra-native FA that we’re bridging to Ethereum.
        //   On Ethereum we create/ensure a wrapped ERC20 via withdrawWrappedAuto.
        // ---------------------------

        let origin_chain_id = cfg.cedra_chain_id_on_eth;
        let origin_asset_bytes32 = cedra_meta_hex_to_bytes32(asset_meta_hex)?;

        // Simple naming for now: prefix with "Cedra " / "C"
        // You can make this nicer (pull metadata name/symbol via another Cedra view).
        let short = if asset_meta_hex.len() > 10 {
            &asset_meta_hex[asset_meta_hex.len() - 10..]
        } else {
            asset_meta_hex
        };
        let name = format!("Cedra Asset {}", short);
        let symbol = format!("C{}", &short[short.len().saturating_sub(4)..]);

        info!(
            "→ Classified as NATIVE Cedra asset. Calling withdrawWrappedAuto with origin_chain_id={} origin_asset={}",
            origin_chain_id, asset_meta_hex
        );

        let mut call = bridge.withdraw_wrapped_auto(
            origin_chain_id,
            origin_asset_bytes32,
            eth_recipient,
            amount,
            nonce_u256,
            name,
            symbol,
        );
        let gas_estimate = call.estimate_gas().await?;
        let gas_limit = gas_estimate * 120u32 / 100u32;
        let call = call.gas(gas_limit);

        let pending_tx = call.send().await?;
        info!(
            "→ Sent withdrawWrappedAuto() tx to Ethereum: {:?}",
            pending_tx.tx_hash()
        );

        match pending_tx.await {
            Ok(Some(receipt)) => {
                info!(
                    "✓ withdrawWrappedAuto confirmed, status: {:?}, gasUsed: {:?}",
                    receipt.status, receipt.gas_used
                );
            }
            Ok(None) => {
                warn!("withdrawWrappedAuto tx pending or dropped (no receipt yet)");
            }
            Err(e) => {
                warn!(
                    "error waiting for withdrawWrappedAuto receipt: {e:?}"
                );
            }
        }
    }

    Ok(())
}

// -------------------- Cedra view helpers --------------------

/// Call `0x...::bridge::is_wrapped_asset(meta_addr)` via /v1/view
async fn cedra_is_wrapped_asset(
    cedra_rest_url: &str,
    cedra_bridge_addr: &str,
    meta_addr_hex: &str,
) -> Result<bool> {
    let func = format!("{}::bridge::is_wrapped_asset", cedra_bridge_addr);
    let url = format!("{}/v1/view", cedra_rest_url);

    let body = serde_json::json!({
        "function": func,
        "type_arguments": [],
        "arguments": [ meta_addr_hex ],
    });

    let client = reqwest::Client::new();
    let resp = client.post(url).json(&body).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Cedra view is_wrapped_asset error {}: {}",
            status,
            text
        ));
    }

    let v: Value = resp.json().await?;
    let arr = v.as_array().ok_or_else(|| anyhow!("view result not array"))?;
    let b = arr
        .get(0)
        .and_then(|x| x.as_bool())
        .ok_or_else(|| anyhow!("view result[0] not bool"))?;
    Ok(b)
}

/// Call `0x...::bridge::origin_of_wrapped(meta_addr): vector<u8>`
/// and return it as hex string "0x..."
async fn cedra_origin_token_of_wrapped(
    cedra_rest_url: &str,
    cedra_bridge_addr: &str,
    meta_addr_hex: &str,
) -> Result<String> {
    let func = format!("{}::bridge::origin_of_wrapped", cedra_bridge_addr);
    let url = format!("{}/v1/view", cedra_rest_url);

    let body = serde_json::json!({
        "function": func,
        "type_arguments": [],
        "arguments": [ meta_addr_hex ],
    });

    let client = reqwest::Client::new();
    let resp = client.post(url).json(&body).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Cedra view origin_of_wrapped error {}: {}",
            status,
            text
        ));
    }

    let v: Value = resp.json().await?;
    let arr = v.as_array().ok_or_else(|| anyhow!("view result not array"))?;
    let s = arr
        .get(0)
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("view result[0] not string"))?;

    Ok(s.to_string())
}

// -------------------- ETH helpers --------------------

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

/// For vector<u8> (hex) -> Address, with left-padding to 20 bytes.
fn bytes_vec_to_eth_address(s: &str) -> Result<Address> {
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

    let full = format!("0x{}", stripped);
    Ok(Address::from_str(&full)?)
}

/// Cedra-origin L1 token (vector<u8>) -> Ethereum token address
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

/// Convert Cedra `address` hex (0x...) into [u8;32] for bytes32 on Ethereum.
fn cedra_meta_hex_to_bytes32(s: &str) -> Result<[u8; 32]> {
    let mut v = s.trim().to_string();
    if v.starts_with("0x") || v.starts_with("0X") {
        v = v[2..].to_string();
    }
    // ensure even length
    if v.len() % 2 != 0 {
        v = format!("0{}", v);
    }

    let mut bytes = Vec::with_capacity(v.len() / 2);
    for i in (0..v.len()).step_by(2) {
        let b = u8::from_str_radix(&v[i..i + 2], 16)
            .map_err(|e| anyhow!("invalid hex {} at {}: {}", s, i, e))?;
        bytes.push(b);
    }

    if bytes.len() > 32 {
        return Err(anyhow!(
            "Cedra meta address too long ({} bytes) for bytes32",
            bytes.len()
        ));
    }

    let mut out = [0u8; 32];
    let start = 32 - bytes.len();
    out[start..].copy_from_slice(&bytes);
    Ok(out)
}