// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cedra_cli::validator::generate_blob, smoke_test_environment::SwarmBuilder,
    utils::get_current_version,
};
use cedra::test::CliTestFramework;
use cedra_forge::{NodeExt, Swarm, SwarmExt};
use cedra_rest_client::Client;
use cedra_types::on_chain_config::{
    BlockGasLimitType, ExecutionConfigV4, OnChainExecutionConfig, TransactionDeduperType,
    TransactionShufflerType,
};
use std::{sync::Arc, time::Duration};

const MAX_WAIT_SECS: u64 = 30;

#[tokio::test]
async fn fallback_test() {
    let swarm = SwarmBuilder::new_local(1)
        .with_init_config(Arc::new(|_, config, _| {
            config.api.failpoints_enabled = true;
            config.execution.discard_failed_blocks = true;
        }))
        .with_cedra()
        .build()
        .await;

    swarm
        .wait_for_all_nodes_to_catchup_to_epoch(2, Duration::from_secs(60))
        .await
        .expect("Epoch 2 taking too long to come!");

    let client = swarm.validators().next().unwrap().rest_client();

    client
        .set_failpoint(
            "cedra_vm::vm_wrapper::execute_transaction".to_string(),
            "100%return".to_string(),
        )
        .await
        .unwrap();

    for _i in 0..1 {
        let version_milestone_0 = get_current_version(&client).await;
        let version_milestone_1 = version_milestone_0 + 5;
        println!("Current version: {}, the chain should tolerate discarding failed blocks, waiting for {}.", version_milestone_0, version_milestone_1);
        swarm
            .wait_for_all_nodes_to_catchup_to_version(version_milestone_1, Duration::from_secs(30))
            .await
            .expect("milestone 1 taking too long");
    }
}

async fn update_execution_config(
    cli: &CliTestFramework,
    root_cli_index: usize,
    new_execution_config: OnChainExecutionConfig,
) {
    let update_execution_config_script = format!(
        r#"
    script {{
        use cedra_framework::cedra_governance;
        use cedra_framework::execution_config;
        fun main(core_resources: &signer) {{
            let framework_signer = cedra_governance::get_signer_testnet_only(core_resources, @0000000000000000000000000000000000000000000000000000000000000001);
            let config_bytes = {};
            execution_config::set_for_next_epoch(&framework_signer, config_bytes);
            cedra_governance::force_end_epoch(&framework_signer);
        }}
    }}
    "#,
        generate_blob(&bcs::to_bytes(&new_execution_config).unwrap())
    );
    cli.run_script(root_cli_index, &update_execution_config_script)
        .await
        .unwrap();
}

async fn get_last_non_reconfig_block_ending_txn_name(rest_client: &Client) -> Option<&'static str> {
    let txns = rest_client
        .get_transactions_bcs(None, Some(10))
        .await
        .unwrap()
        .into_inner();
    let txn_names = txns
        .into_iter()
        .filter(|txn| txn.transaction.is_non_reconfig_block_ending())
        .map(|txn| txn.transaction.type_name())
        .collect::<Vec<_>>();
    println!("{:?}", txn_names);
    txn_names.last().copied()
}

#[tokio::test]
async fn block_epilogue_upgrade_test() {
    let (swarm, mut cli, _faucet) = SwarmBuilder::new_local(2)
        .with_cedra()
        // Start with V1
        .with_init_genesis_config(Arc::new(|genesis_config| {
            genesis_config.execution_config = OnChainExecutionConfig::V4(ExecutionConfigV4 {
                transaction_shuffler_type: TransactionShufflerType::NoShuffling,
                block_gas_limit_type: BlockGasLimitType::NoLimit,
                transaction_deduper_type: TransactionDeduperType::TxnHashAndAuthenticatorV1,
            });
        }))
        .build_with_cli(0)
        .await;

    swarm
        .wait_for_all_nodes_to_catchup_to_future(Duration::from_secs(MAX_WAIT_SECS), 8)
        .await
        .unwrap();

    let rest_client = swarm.validators().next().unwrap().rest_client();

    assert_eq!(
        get_last_non_reconfig_block_ending_txn_name(&rest_client).await,
        Some("state_checkpoint")
    );

    for _ in 0..3 {
        let root_cli_index = cli.add_account_with_address_to_cli(
            swarm.root_key(),
            swarm.chain_info().root_account().address(),
        );

        let current_execution_config =
            crate::utils::get_current_execution_config(&rest_client).await;
        match current_execution_config {
            OnChainExecutionConfig::V4(inner) => {
                assert!(!inner.block_gas_limit_type.add_block_limit_outcome_onchain())
            },
            _ => panic!("Unexpected execution config"),
        };

        // Enable BlockEpilogue
        let mut block_gas_limit = BlockGasLimitType::default_for_genesis();
        match &mut block_gas_limit {
            BlockGasLimitType::ComplexLimitV1 {
                add_block_limit_outcome_onchain,
                ..
            } => *add_block_limit_outcome_onchain = true,
            _ => panic!(),
        };
        let new_execution_config = OnChainExecutionConfig::V4(ExecutionConfigV4 {
            transaction_shuffler_type: TransactionShufflerType::NoShuffling,
            block_gas_limit_type: block_gas_limit,
            transaction_deduper_type: TransactionDeduperType::TxnHashAndAuthenticatorV1,
        });
        update_execution_config(&cli, root_cli_index, new_execution_config).await;

        swarm
            .wait_for_all_nodes_to_catchup_to_future(Duration::from_secs(MAX_WAIT_SECS), 8)
            .await
            .unwrap();
        assert_eq!(
            get_last_non_reconfig_block_ending_txn_name(&rest_client).await,
            Some("block_epilogue")
        );

        let current_execution_config =
            crate::utils::get_current_execution_config(&rest_client).await;
        match current_execution_config {
            OnChainExecutionConfig::V4(inner) => {
                assert!(inner.block_gas_limit_type.add_block_limit_outcome_onchain())
            },
            _ => panic!("Unexpected execution config"),
        };

        // Disable BlockEpilogue
        let new_execution_config = OnChainExecutionConfig::V4(ExecutionConfigV4 {
            transaction_shuffler_type: TransactionShufflerType::NoShuffling,
            block_gas_limit_type: BlockGasLimitType::NoLimit,
            transaction_deduper_type: TransactionDeduperType::TxnHashAndAuthenticatorV1,
        });
        update_execution_config(&cli, root_cli_index, new_execution_config).await;

        swarm
            .wait_for_all_nodes_to_catchup_to_future(Duration::from_secs(MAX_WAIT_SECS), 8)
            .await
            .unwrap();

        assert_eq!(
            get_last_non_reconfig_block_ending_txn_name(&rest_client).await,
            Some("state_checkpoint")
        );
    }
}
