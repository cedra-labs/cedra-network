use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct BridgeRelayersConfig {
    pub cedra_to_eth: Option<CedraToEthRelayerNodeConfig>,
    pub eth_to_cedra: Option<EthToCedraRelayerNodeConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CedraToEthRelayerNodeConfig {
    pub enabled: bool,

    // Cedra side
    pub cedra_rest_url: String,
    pub cedra_bridge_address: String,
    pub cedra_start_version: u64,          // fallback only
    pub cedra_chain_id_on_eth: u16,

    // Cursor persistence
    pub postgres_url: String,
    pub relayer_name: String,              // e.g. "cedra_to_eth_safe"
    pub start_from_latest_if_empty: bool,  // if relayer_status row absent -> head+1

    // Ethereum side
    pub eth_rpc_url: String,
    pub eth_bridge_address: String,
    pub eth_chain_id: u64,
    pub poll_interval_ms: u64,
    pub eth_private_key: String,

    pub safe_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EthToCedraRelayerNodeConfig {
    pub enabled: bool,
    pub eth_rpc_url: String,
    pub eth_bridge_address: String,
    pub eth_start_block: Option<u64>,

    pub cedra_rest_url: String,
    pub cedra_chain_id: u8,
    pub cedra_private_key: String,
    pub cedra_account_address: String,
    pub cedra_bridge_module_address: String,
    pub cedra_multisig_address: String,

    pub cedra_gas_unit_price: u64,
    pub cedra_max_gas: u64,
}