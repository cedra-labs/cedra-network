use anyhow::{anyhow, bail, Context, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;
use tokio_postgres::{Client, NoTls};
use tracing::{info, warn};

abigen!(EthBridge, "../Eth.Bridge.abi.json");

abigen!(
    GnosisSafe,
    r#"[
function nonce() view returns (uint256)
function getThreshold() view returns (uint256)
function getOwners() view returns (address[])
function approvedHashes(address owner, bytes32 hash) view returns (uint256)
function approveHash(bytes32 hashToApprove)
function getTransactionHash(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,uint256 _nonce) view returns (bytes32)
function execTransaction(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,bytes signatures) payable returns (bool success)
]"#
);

const L1_ETH_BYTES_STRIPPED: &str = "0000000000000000000000000000000000000000";

#[derive(Debug, Deserialize)]
struct BridgeEventRow {
    transaction_version: i64,
    event_index: i64,
    kind: String,
    asset: String,
    from_address: Option<String>,
    to_address: Option<String>,
    remote_recipient: Option<String>,
    // IMPORTANT: NUMERIC(39,0) => keep as strings, parse later
    amount: String,
    nonce: String,
}

#[derive(Clone, Debug)]
pub struct WithdrawRelayerConfig {
    // Cedra side
    pub cedra_rest_url: String,
    pub cedra_bridge_address: String,
    pub cedra_chain_id_on_eth: u16,

    // Cursor persistence
    pub postgres_url: String,
    pub relayer_name: String,              // e.g. "cedra_to_eth_safe"
    pub cedra_start_version: u64,          // fallback bootstrap
    pub start_from_latest_if_empty: bool,  // if relayer_status empty -> start at head+1

    // Ethereum side
    pub eth_rpc_url: String,
    pub eth_bridge_address: Address,
    pub eth_chain_id: u64,
    pub poll_interval_ms: u64,
    pub eth_private_key: String,

    pub safe_address: Address,
}

pub async fn run_with_config(cfg: WithdrawRelayerConfig) -> Result<()> {
    // --- Ethereum side ---
    let provider = Provider::<Http>::try_from(cfg.eth_rpc_url.clone())?;
    let wallet = LocalWallet::from_str(&cfg.eth_private_key)?.with_chain_id(cfg.eth_chain_id);
    let provider = SignerMiddleware::new(provider, wallet);
    let provider = std::sync::Arc::new(provider);

    let bridge = EthBridge::new(cfg.eth_bridge_address, provider.clone());
    let safe = GnosisSafe::new(cfg.safe_address, provider.clone());

    let t = safe.get_threshold().call().await.context("Safe not callable / wrong address")?;
    let owners = safe.get_owners().call().await.context("Safe owners not callable")?;
    info!("Safe OK: threshold={} owners={}", t, owners.len());

    let relayer_eoa = provider.address();

    // --- Postgres ---
    let pg = connect_pg(&cfg.postgres_url).await?;
    acquire_relayer_lock(&pg, &cfg.relayer_name).await?;

    // cursor init
    let mut cursor_version: i64 = match load_last_success_version(&pg, &cfg.relayer_name).await? {
        Some(last) => last + 1,
        None => {
            if cfg.start_from_latest_if_empty {
                db_head_version(&pg).await? + 1
            } else {
                cfg.cedra_start_version as i64
            }
        }
    };

    info!("Starting Cedra→Eth (via Safe) relayer...");
    info!("Cedra REST: {}", cfg.cedra_rest_url);
    info!("Cedra bridge addr: {}", cfg.cedra_bridge_address);
    info!("Postgres: {}", cfg.postgres_url);
    info!("Relayer name: {}", cfg.relayer_name);
    info!("Cursor init = {}", cursor_version);
    info!("Eth bridge addr: {:?}", cfg.eth_bridge_address);
    info!("Safe addr: {:?}", cfg.safe_address);
    info!("Relayer EOA (Safe owner key): {:?}", relayer_eoa);

    loop {
        let events = match poll_bridge_deposits_once_db(&pg, cursor_version, 100).await {
            Ok(v) => v,
            Err(e) => {
                warn!("poll_bridge_deposits_once_db error: {e:?}");
                sleep(Duration::from_millis(cfg.poll_interval_ms)).await;
                continue;
            }
        };

        for ev in events {
            // kind is already filtered to deposit in SQL, but keep it safe
            if ev.kind.to_lowercase() != "deposit" {
                cursor_version = ev.transaction_version + 1;
                continue;
            }

            let version_u64 = ev.transaction_version as u64;

            match handle_deposit_event_via_safe(&ev, &cfg, &bridge, &safe, relayer_eoa).await {
                Ok(SafeHandleOutcome::ExecutedOrNoop) => {
                    info!("[Cedra v{}] processed OK (executed or already done).", version_u64);

                    // persist cursor
                    save_last_success_version(&pg, &cfg.relayer_name, ev.transaction_version).await?;

                    cursor_version = ev.transaction_version + 1;
                }
                Ok(SafeHandleOutcome::WaitingForApprovals) => {
                    info!("[Cedra v{}] waiting for approvals; NOT advancing.", version_u64);
                    break;
                }
                Err(e) => {
                    warn!("[Cedra v{}] error processing event: {:#}", version_u64, e);
                    break;
                }
            }
        }

        sleep(Duration::from_millis(cfg.poll_interval_ms)).await;
    }
}

async fn poll_bridge_deposits_once_db(pg: &Client, from_version: i64, limit: i64) -> Result<Vec<BridgeEventRow>> {
    let rows = pg.query(
        r#"
        SELECT
          transaction_version,
          event_index,
          kind,
          asset,
          from_address,
          to_address,
          remote_recipient,
          amount::text AS amount_txt,
          nonce::text  AS nonce_txt
        FROM bridge_events
        WHERE kind = 'deposit'
          AND transaction_version >= $1
        ORDER BY transaction_version ASC, event_index ASC
        LIMIT $2
        "#,
        &[&from_version, &limit],
    ).await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let amount_txt: String = r.get("amount_txt");
        let nonce_txt: String = r.get("nonce_txt");

        out.push(BridgeEventRow {
            transaction_version: r.get("transaction_version"),
            event_index: r.get("event_index"),
            kind: r.get("kind"),
            asset: r.get("asset"),
            from_address: r.get("from_address"),
            to_address: r.get("to_address"),
            remote_recipient: r.get("remote_recipient"),
            amount: amount_txt,
            nonce: nonce_txt,
        });
    }

    Ok(out)
}

enum SafeHandleOutcome {
    ExecutedOrNoop,
    WaitingForApprovals,
}

async fn handle_deposit_event_via_safe(
    ev: &BridgeEventRow,
    cfg: &WithdrawRelayerConfig,
    bridge: &EthBridge<SignerMiddleware<Provider<Http>, LocalWallet>>,
    safe: &GnosisSafe<SignerMiddleware<Provider<Http>, LocalWallet>>,
    relayer_owner: Address,
) -> Result<SafeHandleOutcome> {
    let version = ev.transaction_version as u64;
    let asset_meta_hex = &ev.asset;

    let remote_recipient_hex = ev
        .remote_recipient
        .as_deref()
        .ok_or_else(|| anyhow!("Deposit.remote_recipient missing"))?;
    let eth_recipient = bytes_vec_to_eth_address(remote_recipient_hex)?;

    // SAFE: parse NUMERIC decimal string into U256
    let amount = U256::from_dec_str(ev.amount.trim())
        .map_err(|e| anyhow!("bad amount '{}' (dec): {e}", ev.amount))?;

    info!(
        "[Cedra v{}] Deposit: asset_meta={}, eth_to={:?}, amount={}",
        version, asset_meta_hex, eth_recipient, amount
    );

    let src_nonce_u256 = U256::from_dec_str(ev.nonce.trim())
        .map_err(|e| anyhow!("bad nonce '{}' (dec): {e}", ev.nonce))?;

    if bridge.processed_cedra_nonces(src_nonce_u256).call().await? {
        info!("Already processed srcNonce={} on ETH bridge; skipping", src_nonce_u256);
        return Ok(SafeHandleOutcome::ExecutedOrNoop);
    }

    let is_wrapped = cedra_is_wrapped_asset(&cfg.cedra_rest_url, &cfg.cedra_bridge_address, asset_meta_hex).await?;

    let (to, data): (Address, Bytes) = if is_wrapped {
        let origin_token_hex =
            cedra_origin_token_of_wrapped(&cfg.cedra_rest_url, &cfg.cedra_bridge_address, asset_meta_hex).await?;
        let token_addr = l1_token_bytes_to_eth_token(&origin_token_hex)?;
        info!("→ WRAPPED (ETH-origin). L1 token = {:?}", token_addr);

        let call = bridge.withdraw(eth_recipient, token_addr, amount, src_nonce_u256);
        let calldata = call.calldata().ok_or_else(|| anyhow!("withdraw calldata missing"))?;
        (cfg.eth_bridge_address, calldata)
    } else {
        let origin_chain_id = cfg.cedra_chain_id_on_eth;
        let origin_asset_bytes32 = cedra_meta_hex_to_bytes32(asset_meta_hex)?;

        let short = if asset_meta_hex.len() > 10 { &asset_meta_hex[asset_meta_hex.len() - 10..] } else { asset_meta_hex };
        let name = format!("Cedra Asset {}", short);
        let symbol = format!("C{}", &short[short.len().saturating_sub(4)..]);

        info!("→ NATIVE Cedra asset. withdrawWrappedAuto(origin_chain_id={}, origin_asset={})", origin_chain_id, asset_meta_hex);

        let call = bridge.withdraw_wrapped_auto(origin_chain_id, origin_asset_bytes32, eth_recipient, amount, name, symbol, src_nonce_u256);
        let calldata = call.calldata().ok_or_else(|| anyhow!("withdrawWrappedAuto calldata missing"))?;
        (cfg.eth_bridge_address, calldata)
    };

    if let Err(sim_err) = simulate_as_sender(bridge, cfg.safe_address, &data).await {
        let msg = format!("{:?}", sim_err);
        if msg.to_lowercase().contains("already")
            || msg.to_lowercase().contains("processed")
            || msg.to_lowercase().contains("used nonce")
        {
            warn!("Bridge call seems already processed; skipping Safe submit. err={:?}", sim_err);
            return Ok(SafeHandleOutcome::ExecutedOrNoop);
        }
        return Err(anyhow!("bridge simulation failed: {msg}"));
    }

    let outcome = handle_via_safe(safe, relayer_owner, to, U256::zero(), data).await?;
    Ok(outcome)
}

// ----------------- Safe flow helpers (unchanged) -----------------

async fn handle_via_safe(
    safe: &GnosisSafe<SignerMiddleware<Provider<Http>, LocalWallet>>,
    this_owner: Address,
    to: Address,
    value: U256,
    data: Bytes,
) -> Result<SafeHandleOutcome> {
    let operation: u8 = 0;
    let safe_tx_gas = U256::zero();
    let base_gas = U256::zero();
    let gas_price = U256::zero();
    let gas_token = Address::zero();
    let refund_receiver = Address::zero();

    let safe_nonce = safe.nonce().call().await.context("safe.nonce() failed")?;

    let safe_tx_hash = safe
        .get_transaction_hash(
            to, value, data.clone(), operation,
            safe_tx_gas, base_gas, gas_price,
            gas_token, refund_receiver, safe_nonce,
        )
        .call()
        .await
        .context("safe.getTransactionHash failed")?;

    info!("🔐 Safe nonce={} safeTxHash={:?} to={:?} value={} data_len={}", safe_nonce, safe_tx_hash, to, value, data.0.len());

    let already_approved = safe
        .approved_hashes(this_owner, safe_tx_hash)
        .call()
        .await
        .context("safe.approvedHashes(this_owner,hash) failed")?
        != U256::zero();

    // --- approveHash ---
    if !already_approved {
        info!("📝 approveHash({:?}) as owner {:?}", safe_tx_hash, this_owner);

        // FIX: keep the call builder alive
        let approve_call = safe.approve_hash(safe_tx_hash);
        let pending = approve_call
            .send()
            .await
            .context("approveHash send failed")?;

        let _ = pending.await;
    } else {
        info!("✅ owner {:?} already approved hash {:?}", this_owner, safe_tx_hash);
    }

    let owners = safe.get_owners().call().await.context("safe.getOwners failed")?;
    let threshold = safe.get_threshold().call().await.context("safe.getThreshold failed")?;
    let threshold_usize: usize = threshold.as_usize();
    if threshold_usize == 0 { bail!("Safe threshold is 0 (invalid Safe config)"); }

    let mut approved_others: Vec<Address> = Vec::new();
    for o in owners.iter().copied() {
        if o == this_owner { continue; }
        let ok = safe
            .approved_hashes(o, safe_tx_hash)
            .call()
            .await
            .with_context(|| format!("approvedHashes({:?},{:?}) failed", o, safe_tx_hash))?
            != U256::zero();
        if ok { approved_others.push(o); }
    }

    let needed_other = threshold_usize.saturating_sub(1);
    if approved_others.len() < needed_other {
        info!("⏳ Not enough approvals yet: have_other={} need_other={} threshold={}", approved_others.len(), needed_other, threshold_usize);
        return Ok(SafeHandleOutcome::WaitingForApprovals);
    }

    approved_others.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let mut signers: Vec<Address> = Vec::with_capacity(threshold_usize);
    signers.push(this_owner);
    signers.extend(approved_others.into_iter().take(needed_other));
    signers.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    let signatures = build_approved_hash_signatures(&signers);

    info!("🚀 Executing Safe tx with {} signatures (threshold={})", signers.len(), threshold_usize);

    let call = safe.exec_transaction(
        to,
        value,
        data,
        operation,
        safe_tx_gas,
        base_gas,
        gas_price,
        gas_token,
        refund_receiver,
        signatures,
    );

    let gas_est = call
        .estimate_gas()
        .await
        .context("safe.execTransaction estimate_gas failed")?;

    // FIX: keep the call builder alive (don’t chain .gas(...).send() on a temporary)
    let exec_call = call.gas(gas_est * 120u32 / 100u32);
    let pending = exec_call
        .send()
        .await
        .context("safe.execTransaction send failed")?;

    info!("→ Sent Safe execTransaction tx: {:?}", pending.tx_hash());

    match pending.await {
        Ok(Some(rcpt)) => {
            info!("✓ Safe exec confirmed: status={:?} gasUsed={:?}", rcpt.status, rcpt.gas_used);
        }
        Ok(None) => warn!("Safe exec tx pending/dropped (no receipt yet)"),
        Err(e) => warn!("error waiting for Safe exec receipt: {e:?}"),
    }

    Ok(SafeHandleOutcome::ExecutedOrNoop)
}

fn build_approved_hash_signatures(owners_sorted: &[Address]) -> Bytes {
    let mut out: Vec<u8> = Vec::with_capacity(owners_sorted.len() * 65);
    for owner in owners_sorted {
        let mut r = [0u8; 32];
        r[12..32].copy_from_slice(owner.as_bytes());
        out.extend_from_slice(&r);
        out.extend_from_slice(&[0u8; 32]);
        out.push(1u8);
    }
    Bytes::from(out)
}

async fn simulate_as_sender(
    bridge: &EthBridge<SignerMiddleware<Provider<Http>, LocalWallet>>,
    safe_address: Address,
    calldata: &Bytes,
) -> Result<()> {
    let tx: TypedTransaction = TransactionRequest::new()
        .to(bridge.address())
        .from(safe_address)
        .data(calldata.clone())
        .into();

    bridge.client().call(&tx, None).await?;
    Ok(())
}

// -------------------- Cedra view helpers --------------------

async fn cedra_is_wrapped_asset(cedra_rest_url: &str, cedra_bridge_addr: &str, meta_addr_hex: &str) -> Result<bool> {
    let func = format!("{}::bridge::is_wrapped_asset", cedra_bridge_addr);
    let url = format!("{}/v1/view", cedra_rest_url);

    let body = json!({
        "function": func,
        "type_arguments": [],
        "arguments": [ meta_addr_hex ],
    });

    let resp = reqwest::Client::new().post(url).json(&body).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Cedra view is_wrapped_asset error {}: {}", status, text));
    }

    let v: Value = resp.json().await?;
    let arr = v.as_array().ok_or_else(|| anyhow!("view result not array"))?;
    let b = arr.get(0).and_then(|x| x.as_bool()).ok_or_else(|| anyhow!("view result[0] not bool"))?;
    Ok(b)
}

async fn cedra_origin_token_of_wrapped(cedra_rest_url: &str, cedra_bridge_addr: &str, meta_addr_hex: &str) -> Result<String> {
    let func = format!("{}::bridge::origin_of_wrapped", cedra_bridge_addr);
    let url = format!("{}/v1/view", cedra_rest_url);

    let body = json!({
        "function": func,
        "type_arguments": [],
        "arguments": [ meta_addr_hex ],
    });

    let resp = reqwest::Client::new().post(url).json(&body).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Cedra view origin_of_wrapped error {}: {}", status, text));
    }

    let v: Value = resp.json().await?;
    let arr = v.as_array().ok_or_else(|| anyhow!("view result not array"))?;
    let s = arr.get(0).and_then(|x| x.as_str()).ok_or_else(|| anyhow!("view result[0] not string"))?;
    Ok(s.to_string())
}

// -------------------- ETH helpers --------------------

fn bytes_vec_to_eth_address(s: &str) -> Result<Address> {
    let mut v = s.trim().to_string();
    if !v.starts_with("0x") { v = format!("0x{}", v); }
    let without_0x = &v[2..];
    let stripped = if without_0x.len() < 40 { format!("{:0>40}", without_0x) } else { without_0x.to_string() };
    Ok(Address::from_str(&format!("0x{}", stripped))?)
}

fn l1_token_bytes_to_eth_token(s: &str) -> Result<Address> {
    let mut v = s.trim().to_string();
    if !v.starts_with("0x") { v = format!("0x{}", v); }
    let without_0x = &v[2..];
    let stripped = if without_0x.len() < 40 { format!("{:0>40}", without_0x) } else { without_0x.to_string() };
    if stripped.eq_ignore_ascii_case(L1_ETH_BYTES_STRIPPED) { return Ok(Address::zero()); }
    Ok(Address::from_str(&format!("0x{}", stripped))?)
}

fn cedra_meta_hex_to_bytes32(s: &str) -> Result<[u8; 32]> {
    let mut v = s.trim().to_string();
    if v.starts_with("0x") || v.starts_with("0X") { v = v[2..].to_string(); }
    if v.len() % 2 != 0 { v = format!("0{}", v); }

    let mut bytes = Vec::with_capacity(v.len() / 2);
    for i in (0..v.len()).step_by(2) {
        let b = u8::from_str_radix(&v[i..i + 2], 16).map_err(|e| anyhow!("invalid hex {} at {}: {}", s, i, e))?;
        bytes.push(b);
    }
    if bytes.len() > 32 { return Err(anyhow!("Cedra meta address too long ({} bytes) for bytes32", bytes.len())); }

    let mut out = [0u8; 32];
    let start = 32 - bytes.len();
    out[start..].copy_from_slice(&bytes);
    Ok(out)
}

// -------------------- Postgres helpers --------------------

async fn connect_pg(pg_url: &str) -> Result<Client> {
    let (client, connection) = tokio_postgres::connect(pg_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            warn!("postgres connection error: {e:?}");
        }
    });
    Ok(client)
}

async fn load_last_success_version(pg: &Client, relayer: &str) -> Result<Option<i64>> {
    let row = pg
        .query_opt("SELECT last_success_version FROM relayer_status WHERE relayer = $1", &[&relayer])
        .await?;
    Ok(row.map(|r| r.get::<_, i64>(0)))
}

async fn save_last_success_version(pg: &Client, relayer: &str, v: i64) -> Result<()> {
    pg.execute(
        r#"
        INSERT INTO relayer_status(relayer, last_success_version, updated_at)
        VALUES ($1, $2, now())
        ON CONFLICT (relayer)
        DO UPDATE SET last_success_version = EXCLUDED.last_success_version,
                      updated_at = now()
        "#,
        &[&relayer, &v],
    ).await?;
    Ok(())
}

async fn db_head_version(pg: &Client) -> Result<i64> {
    let row = pg
        .query_one("SELECT COALESCE(MAX(transaction_version), 0) FROM bridge_events WHERE kind = 'deposit'", &[])
        .await?;
    Ok(row.get::<_, i64>(0))
}

async fn acquire_relayer_lock(pg: &Client, relayer: &str) -> Result<()> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    relayer.hash(&mut hasher);
    let key = hasher.finish() as i64;

    let row = pg.query_one("SELECT pg_try_advisory_lock($1)", &[&key]).await?;
    let ok: bool = row.get(0);
    if !ok {
        bail!("relayer lock not acquired (another instance running?) relayer={}", relayer);
    }
    Ok(())
}