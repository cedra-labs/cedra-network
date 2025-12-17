use anyhow::Result;
use ethers::prelude::*;
use std::env;
use cedra_to_eth_relayer::WithdrawRelayerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();

    let cedra_rest_url = env::var("CEDRA_REST_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let cedra_bridge_address = env::var("CEDRA_BRIDGE_ADDRESS")?;
    let eth_rpc_url = env::var("ETH_RPC_URL")?;
    let eth_bridge_address: Address = env::var("ETH_BRIDGE_ADDRESS")?.parse()?;
    let eth_private_key = env::var("ETH_RELAYER_PRIVATE_KEY")?;
    let safe_address: Address = env::var("SAFE_ADDRESS")?.parse()?;

    let poll_interval_ms: u64 = env::var("POLL_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let cedra_start_version: u64 = env::var("CEDRA_START_VERSION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // NEW: Cedra chain id as encoded on the ETH bridge (originChainId)
    let cedra_chain_id_on_eth: u16 = env::var("CEDRA_CHAIN_ID_ON_ETH")
        .ok()
        .and_then(|s| s.parse().ok())
        // fallback to whatever you actually used when deploying the bridge:
        .unwrap_or(4);

    let cfg = WithdrawRelayerConfig {
        cedra_rest_url,
        cedra_bridge_address,
        cedra_start_version,
        cedra_chain_id_on_eth,
        eth_rpc_url,
        eth_bridge_address,
        eth_chain_id: 11155111, // Sepolia in your case
        poll_interval_ms,
        eth_private_key,
        safe_address
    };

    cedra_to_eth_relayer::run_with_config(cfg).await
}