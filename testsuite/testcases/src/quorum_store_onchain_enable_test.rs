// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{generate_onchain_config_blob, NetworkLoadTest};
use anyhow::Ok;
use cedra::test::CliTestFramework;
use cedra_forge::{NetworkContextSynchronizer, NetworkTest, NodeExt, SwarmExt, Test};
use cedra_sdk::bcs;
use cedra_types::{
    account_config::CORE_CODE_ADDRESS,
    on_chain_config::{ConsensusConfigV1, OnChainConsensusConfig},
};
use async_trait::async_trait;
use log::info;
use std::{sync::Arc, time::Duration};

const MAX_NODE_LAG_SECS: u64 = 360;

pub struct QuorumStoreOnChainEnableTest {}

impl Test for QuorumStoreOnChainEnableTest {
    fn name(&self) -> &'static str {
        "quorum-store reconfig enable test"
    }
}

#[async_trait]
impl NetworkLoadTest for QuorumStoreOnChainEnableTest {
    async fn test(
        &self,
        swarm: Arc<tokio::sync::RwLock<Box<dyn cedra_forge::Swarm>>>,
        _report: &mut cedra_forge::TestReport,
        duration: std::time::Duration,
    ) -> anyhow::Result<()> {
        let faucet_endpoint: reqwest::Url = "http://localhost:8081".parse().unwrap();
        let (rest_client, rest_api_endpoint) = {
            let swarm = swarm.read().await;
            let first_validator = swarm.validators().next().unwrap();
            let rest_client = first_validator.rest_client();
            let rest_api_endpoint = first_validator.rest_api_endpoint();
            (rest_client, rest_api_endpoint)
        };
        let mut cli = CliTestFramework::new(
            rest_api_endpoint,
            faucet_endpoint,
            /*num_cli_accounts=*/ 0,
        )
        .await;

        tokio::time::sleep(duration / 2).await;

        let root_cli_index = {
            let root_account = swarm.read().await.chain_info().root_account();
            cli.add_account_with_address_to_cli(
                root_account.private_key().clone(),
                root_account.address(),
            )
        };

        let current_consensus_config: OnChainConsensusConfig = bcs::from_bytes(
            &rest_client
                .get_account_resource_bcs::<Vec<u8>>(
                    CORE_CODE_ADDRESS,
                    "0x1::consensus_config::ConsensusConfig",
                )
                .await
                .unwrap()
                .into_inner(),
        )
        .unwrap();

        let inner = match current_consensus_config {
            OnChainConsensusConfig::V1(inner) => inner,
            OnChainConsensusConfig::V2(_) => panic!("Unexpected V2 config"),
            _ => unimplemented!(),
        };

        // Change to V2
        let new_consensus_config = OnChainConsensusConfig::V2(ConsensusConfigV1 { ..inner });

        let update_consensus_config_script = format!(
            r#"
    script {{
        use cedra_framework::cedra_governance;
        use cedra_framework::consensus_config;
        fun main(core_resources: &signer) {{
            let framework_signer = cedra_governance::get_signer_testnet_only(core_resources, @0000000000000000000000000000000000000000000000000000000000000001);
            let config_bytes = {};
            consensus_config::set(&framework_signer, config_bytes);
        }}
    }}
    "#,
            generate_onchain_config_blob(&bcs::to_bytes(&new_consensus_config).unwrap())
        );

        cli.run_script_with_default_framework(root_cli_index, &update_consensus_config_script)
            .await?;

        tokio::time::sleep(duration / 2).await;

        // Wait for all nodes to synchronize and stabilize.
        info!("Waiting for the validators to be synchronized.");
        swarm
            .read()
            .await
            .wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_NODE_LAG_SECS))
            .await?;

        Ok(())
    }
}

#[async_trait]
impl NetworkTest for QuorumStoreOnChainEnableTest {
    async fn run<'a>(&self, ctx: NetworkContextSynchronizer<'a>) -> anyhow::Result<()> {
        <dyn NetworkLoadTest>::run(self, ctx).await
    }
}
