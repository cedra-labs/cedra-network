use anyhow::*;
use ethers::prelude::*;
use tokio_stream::StreamExt;
use tracing::{info, warn};
use std::{result::Result::Ok, sync::Arc};
use url::Url;

use aptos_sdk::{
    bcs,
    crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt},
    move_types::{
        ident_str,
        language_storage::ModuleId,
    },
    rest_client::Client as AptosClient,
    transaction_builder::TransactionFactory,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, TransactionPayload},
        LocalAccount,
    },
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
///     nonce: u64,
/// )
fn build_withdraw_tokens_entry(
    bridge_module_addr: AccountAddress,
    meta_addr: AccountAddress,
    to: AccountAddress,
    amount: u64,
    nonce: u64,
) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(bridge_module_addr, ident_str!("bridge").to_owned()),
        ident_str!("withdraw_tokens").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&meta_addr).unwrap(),
            bcs::to_bytes(&to).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
            bcs::to_bytes(&nonce).unwrap(),
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
///     nonce: u64,
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
    nonce: u64,
    meta: &WrappedTokenMetadata,
) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(bridge_module_addr, ident_str!("bridge").to_owned()),
        ident_str!("withdraw_auto_create_wrapped").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&origin_token_20).unwrap(),
            bcs::to_bytes(&to).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
            bcs::to_bytes(&nonce).unwrap(),
            bcs::to_bytes(&meta.name.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&meta.symbol.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&meta.decimals).unwrap(),
            bcs::to_bytes(&meta.icon_uri.clone().into_bytes()).unwrap(),
            bcs::to_bytes(&meta.project_uri.clone().into_bytes()).unwrap(),
        ],
    )
}

/// Main entry point used by your Cedra node.
pub async fn run_with_config(cfg: EthToCedraRelayerConfig) -> Result<()> {
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
    let client = AptosClient::new(cedra_url);

    // parse private key
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
                    EthDepositKind::EthNative {
                        cedra_to,
                        amount,
                        nonce,
                    } => {
                        // origin_token = 20 zero bytes
                        let origin_token_20 = vec![0u8; 20];
                        let meta = metadata_resolver.metadata_for_eth();

                        build_withdraw_auto_create_wrapped_entry(
                            bridge_module_addr,
                            origin_token_20,
                            cedra_to,
                            amount,
                            nonce,
                            &meta,
                        )
                    }

                    EthDepositKind::Erc20Native {
                        origin_token_20,
                        cedra_to,
                        amount,
                        nonce,
                    } => {
                        // origin_token_20 is the Ethereum token address as 20 bytes
                        let eth_addr = Address::from_slice(&origin_token_20);
                        let meta = match metadata_resolver.metadata_for_erc20(eth_addr) {
                            Some(m) => m,
                            None => {
                                warn!(
                                    "No metadata config for ERC20 token {:?}; skipping deposit",
                                    eth_addr
                                );
                                continue;
                            }
                        };

                        build_withdraw_auto_create_wrapped_entry(
                            bridge_module_addr,
                            origin_token_20,
                            cedra_to,
                            amount,
                            nonce,
                            &meta,
                        )
                    }

                    EthDepositKind::CedraOriginWrapped {
                        cedra_meta_addr,
                        cedra_to,
                        amount,
                        nonce,
                    } => {
                        // Cedra-origin asset already exists; just mint/unlock on Cedra.
                        build_withdraw_tokens_entry(
                            bridge_module_addr,
                            cedra_meta_addr,
                            cedra_to,
                            amount,
                            nonce,
                        )
                    }
                };

                let payload = TransactionPayload::EntryFunction(entry);

                // 3. Submit transaction on Cedra chain
                let acct = match client.get_account(account.address()).await {
                    Ok(resp) => resp.into_inner(),
                    Err(e) => {
                        warn!("Failed to fetch Cedra account state: {e:?}");
                        continue;
                    }
                };
                account.set_sequence_number(acct.sequence_number);

                let txn = account.sign_with_transaction_builder(tf.payload(payload));
                let resp = match client.submit(&txn).await {
                    Ok(r) => r.into_inner(),
                    Err(e) => {
                        warn!("Failed to submit Cedra tx: {e:?}");
                        continue;
                    }
                };

                let committed = match client.wait_for_transaction(&resp).await {
                    Ok(r) => r.into_inner(),
                    Err(e) => {
                        warn!("Error waiting for Cedra tx: {e:?}");
                        continue;
                    }
                };

                match committed {
                    aptos_sdk::rest_client::Transaction::UserTransaction(utx) => {
                        if utx.info.success {
                            info!("✅ Cedra withdraw_* OK: {}", utx.info.hash);
                        } else {
                            warn!(
                                "❌ Cedra tx failed: {} -- {:?}",
                                utx.info.vm_status, utx
                            );
                        }
                    }
                    other => warn!("Unexpected Cedra transaction kind: {:?}", other),
                }
            }
            Err(e) => {
                warn!("Deposit event stream error: {e:?}");
            }
        }
    }

    Ok(())
}