use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct BridgeRelayersConfig {
    pub cedra_to_eth: Option<CedraToEthRelayerNodeConfig>,
    pub eth_to_cedra: Option<EthToCedraRelayerNodeConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CedraToEthRelayerNodeConfig {
    pub enabled: bool,
    pub cedra_rest_url: String,
    pub cedra_bridge_address: String,
    pub cedra_start_version: u64,
    pub cedra_chain_id_on_eth: u16,
    pub eth_rpc_url: String,
    pub eth_bridge_address: String,
    pub eth_chain_id: u64,
    pub poll_interval_ms: u64,
    pub eth_private_key: String,
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

    pub cedra_gas_unit_price: u64,
    pub cedra_max_gas: u64,
}