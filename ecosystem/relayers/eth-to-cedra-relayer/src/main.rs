use anyhow::*;
use ethers::prelude::*;
use tokio_stream::StreamExt;
use tracing::{info, warn};
use std::sync::Arc;
use std::result::Result::Ok;
use crate::core::utils::hex;

use aptos_sdk::{
    bcs,
    crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt},
    move_types::{identifier::Identifier, language_storage::ModuleId},
    rest_client::Client as AptosClient,
    transaction_builder::TransactionFactory,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        LocalAccount,
        transaction::{EntryFunction, TransactionPayload},
    },
};

use url::Url;
use std::str::FromStr;

abigen!(
    EthBridge,
    "../Eth.Bridge.abi.json" // keep this file next to the relayer crate's Cargo.toml
);

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();

    // =============== Ethereum side ===============
    let eth_rpc = std::env::var("ETH_RPC_URL")?;
    let provider = Provider::<Http>::try_from(eth_rpc)?;
    let provider = Arc::new(provider); // <-- abigen expects Arc<_>
    let bridge_addr: Address = std::env::var("ETH_BRIDGE_ADDRESS")?.parse()?;
    let bridge = EthBridge::new(bridge_addr, provider.clone());

    let from_block = std::env::var("ETH_START_BLOCK")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
        .map(|h| BlockNumber::Number(U64::from(h)))
        .unwrap_or(BlockNumber::Latest);

    let filter = bridge
        .deposit_filter()
        .from_block(from_block);

    let mut stream = filter
        .stream()
        .await?
        .with_meta();

    info!("Listening for Deposit events from {bridge_addr:?}…");

    // =============== Cedra side ===============
    let cedra_url = Url::parse(&std::env::var("CEDRA_REST_URL")?)?;
    let chain_id = ChainId::new(
        std::env::var("CEDRA_CHAIN_ID")
            .ok()
            .and_then(|x| x.parse::<u8>().ok())
            .unwrap_or(4),
    );
    let client = AptosClient::new(cedra_url);

    // single-signer (your "multisig" profile key)
    let mut priv_hex = std::env::var("CEDRA_PRIVATE_KEY_HEX")?;
    if let Some(stripped) = priv_hex.strip_prefix("ed25519-priv-") {
        priv_hex = stripped.to_string();
    }
    let priv_key = Ed25519PrivateKey::from_encoded_string(&priv_hex)?;
    let account = LocalAccount::new(
        AccountAddress::from_str(&std::env::var("CEDRA_ACCOUNT_ADDRESS")?)?,
        priv_key,
        0,
    );

    // bridge entry function: "<addr>::module::function"
    let bridge_mod_str = std::env::var("CEDRA_BRIDGE_MODULE")?; // e.g. 0x...::bridge::execute_deposit
    let (mod_addr, mod_name, fun_name) = parse_module(&bridge_mod_str)?;
    let module_id = ModuleId::new(mod_addr, Identifier::new(mod_name)?);
    let func_id = Identifier::new(fun_name)?;

    let gas_unit_price = std::env::var("CEDRA_GAS_UNIT_PRICE")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(100);
    let max_gas = std::env::var("CEDRA_MAX_GAS")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(200_000);

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

                // Convert bytes32 -> [u8; 32] -> Cedra AccountAddress
                // Works whether abigen uses H256 or FixedBytes<32>:
                let cedra_to_32: [u8; 32] = event.cedra_to.into();
                let to_l2 = match aptos_sdk::types::account_address::AccountAddress::from_bytes(cedra_to_32) {
                    Ok(addr) => addr,
                    Err(_) => {
                        warn!("Deposit.cedra_to is not a valid 32-byte Cedra address: 0x{}", hex::encode(cedra_to_32));
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

                // Build execute_deposit(l1_token, to, amount, nonce, eth_tx_hash)
                let payload = TransactionPayload::EntryFunction(EntryFunction::new(
                    module_id.clone(),
                    func_id.clone(),
                    vec![],
                    vec![
                        bcs::to_bytes(&l1_token_20)?,
                        bcs::to_bytes(&to_l2)?,
                        bcs::to_bytes(&amount_u64)?,
                        bcs::to_bytes(&nonce_u64)?,
                        bcs::to_bytes(&tx_hash)?,
                    ],
                ));

                // Refresh seq
                let acct = client.get_account(account.address()).await?.into_inner();
                account.set_sequence_number(acct.sequence_number);

                // Submit
                let txn = account.sign_with_transaction_builder(tf.payload(payload));
                let resp = client.submit(&txn).await?;
                let pending = resp.into_inner();
                let committed = client.wait_for_transaction(&pending).await?;
                let tx = committed.into_inner();
                //todo how to act if failed? should eth contract refund tokens?

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

// "0xADDR::module::function"
fn parse_module(s: &str) -> Result<(AccountAddress, &str, &str)> {
    let mut parts = s.split("::");
    let addr = parts
        .next()
        .ok_or_else(|| anyhow!("bad module string"))?
        .parse::<AccountAddress>()?;
    let module = parts.next().ok_or_else(|| anyhow!("missing module"))?;
    let func = parts.next().ok_or_else(|| anyhow!("missing function"))?;
    Ok((addr, module, func))
}