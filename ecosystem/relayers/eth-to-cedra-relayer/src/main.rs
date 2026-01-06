use anyhow::Result;
use ethers::prelude::*;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use cedra_sdk::types::account_address::AccountAddress;
use eth_to_cedra_relayer::{EthToCedraRelayerConfig, SimpleMetadataResolver};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();

    let eth_rpc_url = env::var("ETH_RPC_URL")?;
    let eth_bridge_address: Address = env::var("ETH_BRIDGE_ADDRESS")?.parse()?;
    let eth_start_block = env::var("ETH_START_BLOCK")
        .ok()
        .and_then(|x| x.parse::<u64>().ok());

    let cedra_rest_url = env::var("CEDRA_REST_URL")?;
    let cedra_chain_id = env::var("CEDRA_CHAIN_ID")
        .ok()
        .and_then(|x| x.parse::<u8>().ok())
        .unwrap_or(4);

    let cedra_private_key = env::var("CEDRA_PRIVATE_KEY_HEX")?;
    let cedra_account_address =
        AccountAddress::from_str(&env::var("CEDRA_ACCOUNT_ADDRESS")?)?;

    let cedra_bridge_module_address =
        AccountAddress::from_str(&env::var("CEDRA_BRIDGE_MODULE_ADDRESS")?)?;
    let cedra_multisig_address =
        AccountAddress::from_str(&env::var("CEDRA_MULTISIG_ADDRESS")?)?;

    let cedra_gas_unit_price = env::var("CEDRA_GAS_UNIT_PRICE")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(100);
    let cedra_max_gas = env::var("CEDRA_MAX_GAS")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(200_000);

    let cfg = EthToCedraRelayerConfig {
        eth_rpc_url,
        eth_bridge_address,
        eth_start_block,
        cedra_rest_url,
        cedra_chain_id,
        cedra_private_key,
        cedra_account_address,
        cedra_bridge_module_address,
        cedra_multisig_address,
        cedra_gas_unit_price,
        cedra_max_gas,
        // new field:
        metadata_resolver: Arc::new(SimpleMetadataResolver),
    };

    eth_to_cedra_relayer::run_with_config(cfg).await
}