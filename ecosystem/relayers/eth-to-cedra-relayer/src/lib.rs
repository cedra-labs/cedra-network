use anyhow::*;
use ethers::prelude::*;
use tokio_stream::StreamExt;
use tracing::{info, warn};
use std::{result::Result::Ok, sync::Arc};
use url::Url;
use serde_json::json;

use cedra_sdk::{
    bcs,
    crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt},
    move_types::{ident_str, language_storage::ModuleId},
    transaction_builder::TransactionFactory,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, TransactionPayload, Multisig, MultisigTransactionPayload},
        LocalAccount, CedraCoinType, CoinType
    },
};

use cedra_rest_client::{
    Client as CedraClient,
    cedra_api_types::{ViewRequest, EntryFunctionId},
    Transaction as CedraTransaction,
};

abigen!(
    EthBridge,
    "../Eth.Bridge.abi.json"
);

pub struct SimpleMetadataResolver;

impl MetadataResolver for SimpleMetadataResolver {
    fn metadata_for_eth(&self) -> WrappedTokenMetadata {
        WrappedTokenMetadata {
            // You can tweak these strings later without breaking the bridge logic.
            name: "Cedra Wrapped ETH".to_string(),
            symbol: "CETH".to_string(), // <--- "C" prefix
            decimals: 18,
            icon_uri: "".to_string(),
            project_uri: "".to_string(),
        }
    }

    fn metadata_for_erc20(&self, token: Address) -> Option<WrappedTokenMetadata> {
        let addr_hex = format!("{:#x}", token); // "0x9579..."

        // Short fragment for symbol / name
        let short = addr_hex
            .trim_start_matches("0x")
            .chars()
            .take(6)
            .collect::<String>();

        // Keep name well under typical 32-char limit
        let name = format!("Cedra-wrapped-{}", short.to_uppercase());   // e.g. "Cedra-wrapped-957958"
        let symbol = format!("C{}", short.to_uppercase());  // e.g. "C95795"

        Some(WrappedTokenMetadata {
            name,
            symbol,
            // For your MockERC20 with 6 decimals; if you want to be generic,
            // you could store per-token config or call `decimals()` offchain.
            decimals: 6,
            // If you still want full address somewhere, stash it here:
            icon_uri: addr_hex.clone(),      // or some URL
            project_uri: "".to_string(),
        })
    }
}

/// How we describe the metadata of a wrapped token on Cedra.
#[derive(Clone, Debug)]
pub struct WrappedTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon_uri: String,
    pub project_uri: String,
}

/// Trait the caller must implement to provide metadata for new wrapped tokens.
/// This keeps the core relayer logic clean.
pub trait MetadataResolver: Send + Sync {
    /// Metadata for wrapped ETH on Cedra (origin_token = 20 zero bytes).
    fn metadata_for_eth(&self) -> WrappedTokenMetadata;

    /// Metadata for a wrapped ERC20 on Cedra, given the *origin* ERC20 address.
    /// Return None if this token is not supported.
    fn metadata_for_erc20(&self, token: Address) -> Option<WrappedTokenMetadata>;
}

/// Config for eth→cedra relayer.
#[derive(Clone)]
pub struct EthToCedraRelayerConfig {
    // Ethereum side
    pub eth_rpc_url: String,
    pub eth_bridge_address: Address,
    pub eth_start_block: Option<u64>,

    // Cedra side
    pub cedra_rest_url: String,
    pub cedra_chain_id: u8,

    pub cedra_private_key: String,        // hex or "ed25519-priv-..."
    pub cedra_account_address: AccountAddress,

    pub cedra_bridge_module_address: AccountAddress, // address that has module `bridge::bridge`
    pub cedra_multisig_address: AccountAddress,

    pub cedra_gas_unit_price: u64,
    pub cedra_max_gas: u64,

    /// Resolver for naming/symbol/decimals/URIs of newly created wrapped tokens.
    pub metadata_resolver: Arc<dyn MetadataResolver>,
}

/// Internal classification of an Ethereum Deposit event.
#[derive(Debug, Clone)]
enum EthDepositKind {
    /// depositETH: native ETH locked on Ethereum
    EthNative {
        cedra_to: AccountAddress,
        amount: u64,
        nonce: u64,
    },
    /// depositERC20: native ERC20 locked on Ethereum
    Erc20Native {
        origin_token_20: Vec<u8>, // 20-byte Ethereum token address
        cedra_to: AccountAddress,
        amount: u64,
        nonce: u64,
    },
    /// depositWrapped: Cedra-origin token burned on Ethereum
    CedraOriginWrapped {
        cedra_meta_addr: AccountAddress, // Cedra Metadata address (originAssetAddress)
        cedra_to: AccountAddress,
        amount: u64,
        nonce: u64,
    },
}

/// Classify a Deposit event using on-chain view calls:
/// - token == 0       → EthNative
/// - !isWrappedToken  → Erc20Native
/// - isWrappedToken   → CedraOriginWrapped (use assetMeta to get Cedra origin asset)
async fn classify_deposit<M: Middleware + 'static>(
    bridge: &EthBridge<M>,
    event: DepositFilter,
) -> Result<EthDepositKind> {
    let amount_u64: u64 = event
        .amount
        .try_into()
        .map_err(|_| anyhow!("amount too large for u64"))?;
    let nonce_u64 = event.nonce.as_u64();

    let cedra_to_32: [u8; 32] = event.cedra_to.into();
    let cedra_to =
        AccountAddress::from_bytes(cedra_to_32).context("Deposit.cedra_to is not a valid Cedra address")?;

    // Case 1: ETH native
    if event.token == Address::zero() {
        return Ok(EthDepositKind::EthNative {
            cedra_to,
            amount: amount_u64,
            nonce: nonce_u64,
        });
    }

    // Ask Ethereum bridge if this token is a wrapped Cedra-origin token
    let is_wrapped: bool = bridge
        .is_wrapped_token(event.token)
        .call()
        .await
        .context("isWrappedToken call failed")?;

    if is_wrapped {
        // Case 2: Cedra-origin wrapped token on Ethereum.
        // assetMeta[wrappedToken].assetAddress = Cedra origin asset address (bytes32)
       let origin_asset_bytes32: [u8; 32] = bridge
        .asset_meta(event.token)
        .call()
        .await
        .context("assetMeta call failed")?;

        let cedra_meta_addr = AccountAddress::from_bytes(origin_asset_bytes32)
            .context("invalid originAssetAddress as Cedra address")?;

        return Ok(EthDepositKind::CedraOriginWrapped {
            cedra_meta_addr,
            cedra_to,
            amount: amount_u64,
            nonce: nonce_u64,
        });
    }

    // Case 3: Ethereum-native ERC20 (must be whitelisted isNativeToken[token] == true).
    // Optional extra check:
    // let is_native = bridge.is_native_token(event.token).call().await?;
    // if !is_native { bail!("token neither wrapped nor native"); }

    Ok(EthDepositKind::Erc20Native {
        origin_token_20: event.token.0.to_vec(),
        cedra_to,
        amount: amount_u64,
        nonce: nonce_u64,
    })
}

/// Build Move entry for:
///
/// public entry fun withdraw_tokens(
///     multisig: &signer,
///     meta_addr: address,
///     to: address,
///     amount: u64,
/// )
fn build_withdraw_tokens_entry(
    bridge_module_addr: AccountAddress,
    meta_addr: AccountAddress,
    to: AccountAddress,
    amount: u64,
    src_nonce: u64,
) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(bridge_module_addr, ident_str!("bridge").to_owned()),
        ident_str!("withdraw_tokens").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&meta_addr).unwrap(),
            bcs::to_bytes(&to).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
            bcs::to_bytes(&src_nonce).unwrap(),
        ],
    )
}

/// Build Move entry for:
///
/// public entry fun withdraw_auto_create_wrapped(
///     multisig: &signer,
///     origin_token: vector<u8>,
///     to: address,
///     amount: u64,
///     name: vector<u8>,
///     symbol: vector<u8>,
///     decimals: u8,
///     icon_uri: vector<u8>,
///     project_uri: vector<u8>,
/// )
fn build_withdraw_auto_create_wrapped_entry(
    bridge_module_addr: AccountAddress,
    origin_token_20: Vec<u8>,
    to: AccountAddress,
    amount: u64,
    meta: &WrappedTokenMetadata,
    src_nonce: u64,
) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(bridge_module_addr, ident_str!("bridge").to_owned()),
        ident_str!("withdraw_auto_create_wrapped").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&origin_token_20).unwrap(),
            bcs::to_bytes(&to).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
            bcs::to_bytes(&meta.name.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&meta.symbol.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&meta.decimals).unwrap(),
            bcs::to_bytes(&meta.icon_uri.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&meta.project_uri.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&src_nonce).unwrap(),
        ],
    )
}

/// Main entry point used by your Cedra node.
pub async fn run_with_config(cfg: EthToCedraRelayerConfig) -> Result<()> {
    info!("Starting Eth→Cedra relayer.");
    // =============== Ethereum side ===============
    let provider = Provider::<Http>::try_from(cfg.eth_rpc_url.clone())?;
    let provider = Arc::new(provider);
    let bridge = EthBridge::new(cfg.eth_bridge_address, provider.clone());

    let from_block = cfg
        .eth_start_block
        .map(|h| BlockNumber::Number(U64::from(h)))
        .unwrap_or(BlockNumber::Latest);

    let filter = bridge.deposit_filter().from_block(from_block);
    let mut stream = filter.stream().await?.with_meta();

    info!(
        "Listening for Deposit events from {:?} starting at {:?}…",
        cfg.eth_bridge_address, from_block
    );

    // =============== Cedra side ===============
    let cedra_url = Url::parse(&cfg.cedra_rest_url)?;
    let chain_id = ChainId::new(cfg.cedra_chain_id);
    let client = CedraClient::new(cedra_url);

    // parse private key
    let mut priv_hex = cfg.cedra_private_key.clone();
    if let Some(stripped) = priv_hex.strip_prefix("ed25519-priv-") {
        priv_hex = stripped.to_string();
    }
    let priv_key = Ed25519PrivateKey::from_encoded_string(&priv_hex)?;
    let mut account = LocalAccount::new(cfg.cedra_account_address, priv_key, 0);

    let gas_unit_price = cfg.cedra_gas_unit_price;
    let max_gas = cfg.cedra_max_gas;

    let tf = TransactionFactory::new(chain_id, CedraCoinType::type_tag())
        .with_gas_unit_price(gas_unit_price)
        .with_max_gas_amount(max_gas);

    let metadata_resolver = cfg.metadata_resolver.clone();
    let bridge_module_addr = cfg.cedra_bridge_module_address;

    while let Some(item) = stream.next().await {
        match item {
            Ok((event, meta)) => {
                // event: Deposit(token, from, cedraTo, amount, nonce)
                info!(
                    "⚡ New Deposit on ETH: token={:?}, from={:?}, cedra_to={:?}, amount={}, nonce={}, tx={:?}",
                    event.token,
                    event.from,
                    event.cedra_to,
                    event.amount,
                    event.nonce,
                    meta.transaction_hash,
                );

                // 1. Classify the deposit (ETH / ERC20 native / Cedra-origin wrapped)
                let kind = match classify_deposit(&bridge, event.clone()).await {
                    Ok(k) => k,
                    Err(err) => {
                        warn!("Failed to classify deposit event: {err:?}");
                        continue;
                    }
                };

                // 2. Build the appropriate Cedra EntryFunction
                let entry: EntryFunction = match kind {
                    EthDepositKind::EthNative { cedra_to, amount, nonce } => {
                        let origin_token_20 = vec![0u8; 20];
                        let meta = metadata_resolver.metadata_for_eth();
                        build_withdraw_auto_create_wrapped_entry(
                            bridge_module_addr, origin_token_20, cedra_to, amount, &meta, nonce
                        )
                    }
                    EthDepositKind::Erc20Native { origin_token_20, cedra_to, amount, nonce } => {
                        let eth_addr = Address::from_slice(&origin_token_20);
                        let meta = metadata_resolver
                            .metadata_for_erc20(eth_addr)
                            .ok_or_else(|| anyhow!("No metadata for token"))?;
                        build_withdraw_auto_create_wrapped_entry(
                            bridge_module_addr, origin_token_20, cedra_to, amount, &meta, nonce
                        )
                    }
                    EthDepositKind::CedraOriginWrapped { cedra_meta_addr, cedra_to, amount, nonce } => {
                        build_withdraw_tokens_entry(
                            bridge_module_addr, cedra_meta_addr, cedra_to, amount, nonce
                        )
                    }
                };

                let inner_payload = TransactionPayload::EntryFunction(entry);

                // 1. create multisig transaction if it doesn't exist, else find existing one
                // 2. approve it from this owner
                // 3. if enough approvals → execute it as multisig
                if let Err(err) = handle_via_multisig(
                    &client,
                    &mut account,
                    &tf,
                    cfg.cedra_multisig_address,
                    inner_payload,
                ).await {
                    warn!("❌ handle_via_multisig failed for deposit nonce={}: {:#}", event.nonce, err);
                }
            }
            Err(e) => {
                warn!("Deposit event stream error: {e:?}");
            }
        }
    }

    Ok(())
}

/// -------------
/// HELPERS
/// -------------

async fn view_u64(
    client: &CedraClient,
    function: &str,
    args: Vec<serde_json::Value>,
) -> Result<u64> {
    let req = ViewRequest {
        function: function.parse::<EntryFunctionId>()?,
        type_arguments: vec![],
        arguments: args,
    };

    let resp = client.view(&req, None).await?.into_inner();
    let v = resp
        .get(0)
        .ok_or_else(|| anyhow!("empty view result for {function}"))?;

    let s = v
        .as_str()
        .ok_or_else(|| anyhow!("expected string result for {function}, got {v:?}"))?;

    Ok(s.parse::<u64>()?)
}

/// Fetch a single MultisigTransaction via view.
/// We only care about the raw payload bytes and the sequence number.
#[derive(Debug, Clone)]
struct OnchainMultisigTx {
    pub sequence_number: u64,
    pub payload: Option<Vec<u8>>,
    pub payload_hash: Option<Vec<u8>>,
}

async fn view_multisig_transaction(
    client: &CedraClient,
    multisig: AccountAddress,
    sequence_number: u64,
) -> Result<OnchainMultisigTx> {
    let multisig_str = multisig.to_hex_literal();
    let req = ViewRequest {
        function: "0x1::multisig_account::get_transaction".parse::<EntryFunctionId>()?,
        type_arguments: vec![],
        arguments: vec![json!(multisig_str), json!(sequence_number.to_string())],
    };

    let resp = client.view(&req, None).await?.into_inner();
    let obj = resp
        .get(0)
        .ok_or_else(|| anyhow!("empty view result for get_transaction"))?;

    let decode_opt_bytes = |field: &str| -> Option<Vec<u8>> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .and_then(|hex_str| {
                if hex_str == "0x" {
                    None
                } else {
                    hex::decode(hex_str.trim_start_matches("0x")).ok()
                }
            })
    };

    let payload = decode_opt_bytes("payload");
    let payload_hash = decode_opt_bytes("payload_hash");

    Ok(OnchainMultisigTx {
        sequence_number,
        payload,
        payload_hash,
    })
}

/// Check this owner’s vote (if any) on a given tx.
/// Returns (voted?, approved?).
async fn view_vote_for_owner(
    client: &CedraClient,
    multisig: AccountAddress,
    sequence_number: u64,
    owner: AccountAddress,
) -> Result<(bool, bool)> {
    let multisig_str = multisig.to_hex_literal();
    let owner_str = owner.to_hex_literal();

    let req = ViewRequest {
        function: "0x1::multisig_account::vote".parse::<EntryFunctionId>()?,
        type_arguments: vec![],
        arguments: vec![
            json!(multisig_str),
            json!(sequence_number.to_string()),
            json!(owner_str),
        ],
    };

    let resp = client.view(&req, None).await?.into_inner();
    tracing::info!(
        "🔎 view_vote_for_owner raw resp (msig={}, seq={}, owner={}): {:?}",
        multisig_str,
        sequence_number,
        owner_str,
        resp
    );

    if resp.is_empty() {
        bail!("empty view result for vote");
    }

    // Cedra right now returns: [Bool(true), Bool(true)]
    // But be robust and also support [[true, true]] if that ever appears.
    let (voted_val, approved_val) = if resp.len() == 1 && resp[0].is_array() {
        // Shape: [[true, true]]
        let arr = resp[0]
            .as_array()
            .ok_or_else(|| anyhow!("vote[0] is not array: {:?}", resp[0]))?;
        if arr.len() != 2 {
            bail!("nested vote array len != 2: {:?}", arr);
        }
        (&arr[0], &arr[1])
    } else {
        // Shape: [true, true]
        if resp.len() < 2 {
            bail!("vote result has len < 2: {:?}", resp);
        }
        (&resp[0], &resp[1])
    };

    let voted = voted_val
        .as_bool()
        .ok_or_else(|| anyhow!("voted is not bool: {:?}", voted_val))?;
    let approved = approved_val
        .as_bool()
        .ok_or_else(|| anyhow!("approved is not bool: {:?}", approved_val))?;

    Ok((voted, approved))
}

/// View helper: can *this owner* execute (may imply implicit approval)?
async fn view_can_execute(
    client: &CedraClient,
    multisig: AccountAddress,
    sequence_number: u64,
    owner: AccountAddress,
) -> Result<bool> {
    let multisig_str = multisig.to_hex_literal();
    let owner_str = owner.to_hex_literal();
    let req = ViewRequest {
        function: "0x1::multisig_account::can_execute".parse::<EntryFunctionId>()?,
        type_arguments: vec![],
        arguments: vec![
            json!(owner_str),
            json!(multisig_str),
            json!(sequence_number.to_string()),
        ],
    };
    let resp = client.view(&req, None).await?.into_inner();
    let v = resp.get(0).ok_or_else(|| anyhow!("empty result for can_execute"))?;
    v.as_bool()
        .ok_or_else(|| anyhow!("expected bool from can_execute, got {v:?}"))
}

fn to_msig_payload_bytes(inner: &TransactionPayload) -> Result<Vec<u8>> {
    match inner {
        TransactionPayload::EntryFunction(ef) => {
            let msig = MultisigTransactionPayload::EntryFunction(ef.clone());
            Ok(bcs::to_bytes(&msig)?)
        }
        other => bail!("unsupported inner payload for multisig create_transaction: {:?}", other),
    }
}

/// Returns (sequence_number, created_new?)
async fn find_or_create_multisig_tx(
    client: &CedraClient,
    tf: &TransactionFactory,
    owner: &mut LocalAccount,
    multisig: AccountAddress,
    inner_payload: &TransactionPayload,
) -> Result<(u64, bool)> {
    let inner_bytes = to_msig_payload_bytes(inner_payload)?;

    let multisig_str = multisig.to_hex_literal();

    let last_resolved = view_u64(
        client,
        "0x1::multisig_account::last_resolved_sequence_number",
        vec![json!(multisig_str.clone())],
    ).await?;

    let next_seq = view_u64(
        client,
        "0x1::multisig_account::next_sequence_number",
        vec![json!(multisig_str.clone())],
    ).await?;

    // Scan pending transactions [last_resolved+1, next_seq-1] (max 20 by module)
    for seq in (last_resolved + 1)..next_seq {
        let onchain_tx = view_multisig_transaction(client, multisig, seq).await?;
        if let Some(payload_bytes) = onchain_tx.payload {
            if payload_bytes == inner_bytes {
                return Ok((seq, false)); // already exists
            }
        }
    }

    // Not found → create new multisig transaction with full payload
    let create_payload = TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(AccountAddress::ONE, ident_str!("multisig_account").to_owned()),
        ident_str!("create_transaction").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&multisig)?,
            bcs::to_bytes(&inner_bytes)?, // payload: vector<u8>
        ],
    ));

    let acct = client.get_account(owner.address()).await?.into_inner();
    owner.set_sequence_number(acct.sequence_number);

    let tx = owner.sign_with_transaction_builder(tf.payload(create_payload));
    let pending = client.submit(&tx).await?.into_inner();
    let committed = client.wait_for_transaction(&pending).await?.into_inner();

    let new_next_seq = view_u64(
        client,
        "0x1::multisig_account::next_sequence_number",
        vec![json!(multisig_str)],
    ).await?;
    let seq = new_next_seq - 1;

    match committed {
        CedraTransaction::UserTransaction(utx) => {
            if utx.info.success {
                Ok((seq, true)) // або твоя логіка
            } else {
                Err(anyhow!("create_transaction failed: {}", utx.info.vm_status))
            }
        }
        other => Err(anyhow!("unexpected tx kind for create_transaction: {other:?}")),
    }
}

async fn ensure_approval_for_owner(
    client: &CedraClient,
    tf: &TransactionFactory,
    owner: &mut LocalAccount,
    multisig: AccountAddress,
    sequence_number: u64,
) -> Result<()> {
    let owner_addr = owner.address();

    let (voted, approved) = match view_vote_for_owner(client, multisig, sequence_number, owner_addr).await {
        Ok(v) => {
            tracing::info!(
                "🗳 current vote state (owner={}, msig={}, seq={}): {:?}",
                owner_addr,
                multisig,
                sequence_number,
                v
            );
            v
        }
        Err(e) => {
            tracing::warn!(
                "❌ view_vote_for_owner failed (owner={}, msig={}, seq={}): {:#}",
                owner_addr,
                multisig,
                sequence_number,
                e
            );
            return Err(e);
        }
    };

    if voted && approved {
        tracing::info!(
            "✅ Owner {} already approved multisig tx {}",
            owner_addr,
            sequence_number
        );
        return Ok(());
    }

    tracing::info!(
        "📝 Approving multisig tx {} as owner {}",
        sequence_number,
        owner_addr
    );

    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(AccountAddress::ONE, ident_str!("multisig_account").to_owned()),
        ident_str!("approve_transaction").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&multisig)?,
            bcs::to_bytes(&sequence_number)?,
        ],
    ));

    let acct = client.get_account(owner.address()).await?.into_inner();
    owner.set_sequence_number(acct.sequence_number);

    let tx = owner.sign_with_transaction_builder(tf.payload(payload));
    let pending = client.submit(&tx).await?.into_inner();
    let committed = client.wait_for_transaction(&pending).await?.into_inner();

    if let CedraTransaction::UserTransaction(utx) = committed {
        if utx.info.success {
            Ok(())
        } else {
            Err(anyhow!("approve_transaction failed: {}", utx.info.vm_status))
        }
    } else {
        Err(anyhow!("unexpected tx kind for approve_transaction"))
    }
}

async fn try_execute_multisig_tx(
    client: &CedraClient,
    tf: &TransactionFactory,
    owner: &mut LocalAccount,
    multisig: AccountAddress,
    sequence_number: u64,
    inner_payload: &TransactionPayload,
) -> Result<()> {
    let owner_addr = owner.address();

    let can_exec = match view_can_execute(client, multisig, sequence_number, owner_addr).await {
        Ok(b) => {
            info!(
                "🔍 can_execute(owner={}, multisig={}, seq={}) = {}",
                owner_addr,
                multisig,
                sequence_number,
                b
            );
            b
        }
        Err(e) => {
            warn!(
                "❌ view_can_execute failed (owner={}, msig={}, seq={}): {:#}",
                owner_addr,
                multisig,
                sequence_number,
                e
            );
            return Err(e);
        }
    };
    if !can_exec {
        return Ok(());
    }

    // OLD (BUGGY) PART – RE-SENDING PAYLOAD WITH A DIFFERENT TYPE
    // -----------------------------------------------------------
    // let entry_fn = match inner_payload {
    //     TransactionPayload::EntryFunction(ef) => ef.clone(),
    //     other => {
    //         bail!("Multisig inner payload must be EntryFunction, got {:?}", other);
    //     }
    // };
    //
    // let msig_payload = MultisigTransactionPayload::EntryFunction(entry_fn);
    //
    // let msig = Multisig {
    //     multisig_address: multisig,
    //     transaction_payload: Some(msig_payload),
    // };
    //
    // let outer_payload = TransactionPayload::Multisig(msig);

    // ✅ NEW: for create_transaction (full payload stored on-chain),
    // we MUST NOT re-send payload bytes. Let the on-chain payload be used.
    let msig = Multisig {
        multisig_address: multisig,
        transaction_payload: None, // <--- key change
    };

    let outer_payload = TransactionPayload::Multisig(msig);

    let acct = client.get_account(owner.address()).await?.into_inner();
    owner.set_sequence_number(acct.sequence_number);

    let tx = owner.sign_with_transaction_builder(tf.payload(outer_payload));
    let pending = client.submit(&tx).await?.into_inner();
    let committed = client.wait_for_transaction(&pending).await?.into_inner();

    match committed {
        CedraTransaction::UserTransaction(utx) => {
            if utx.info.success {
                info!(
                    "✅ Multisig execution succeeded: seq={}, hash={}",
                    sequence_number, utx.info.hash
                );
            } else {
                warn!(
                    "❌ Multisig execution failed for seq {}: {}",
                    sequence_number, utx.info.vm_status
                );
            }
        }
        other => {
            warn!("Unexpected tx kind for multisig execute: {:?}", other);
        }
    }

    Ok(())
}

async fn handle_via_multisig(
    client: &CedraClient,
    owner: &mut LocalAccount,
    tf: &TransactionFactory,
    multisig: AccountAddress,
    inner_payload: TransactionPayload,
) -> Result<()> {
    let (seq, created_new) =
        find_or_create_multisig_tx(client, tf, owner, multisig, &inner_payload).await?;

    if created_new {
        info!("📜 Created new multisig tx seq={seq} for this deposit");
    } else {
        info!("📜 Reusing existing multisig tx seq={seq} for this deposit");
    }

    ensure_approval_for_owner(client, tf, owner, multisig, seq).await?;
    try_execute_multisig_tx(client, tf, owner, multisig, seq, &inner_payload).await?;

    Ok(())
}