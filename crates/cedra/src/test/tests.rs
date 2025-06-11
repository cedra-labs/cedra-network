// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_tool::{ArgWithType, FunctionArgType},
    CliResult, Tool,
};
use clap::Parser;
use std::str::FromStr;

/// In order to ensure that there aren't duplicate input arguments for untested CLI commands,
/// we call help on every command to ensure it at least runs
#[tokio::test]
async fn ensure_every_command_args_work() {
    assert_cmd_not_panic(&["cedra"]).await;

    assert_cmd_not_panic(&["cedra", "account"]).await;
    assert_cmd_not_panic(&["cedra", "account", "create", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "account", "create-resource-account", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "account", "fund-with-faucet", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "account", "list", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "account", "lookup-address", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "account", "rotate-key", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "account", "transfer", "--help"]).await;

    assert_cmd_not_panic(&["cedra", "config"]).await;
    assert_cmd_not_panic(&["cedra", "config", "generate-shell-completions", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "config", "init", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "config", "set-global-config", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "config", "show-global-config"]).await;
    assert_cmd_not_panic(&["cedra", "config", "show-profiles"]).await;

    assert_cmd_not_panic(&["cedra", "genesis"]).await;
    assert_cmd_not_panic(&["cedra", "genesis", "generate-genesis", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "genesis", "generate-keys", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "genesis", "generate-layout-template", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "genesis", "set-validator-configuration", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "genesis", "setup-git", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "genesis", "generate-admin-write-set", "--help"]).await;

    assert_cmd_not_panic(&["cedra", "governance"]).await;
    assert_cmd_not_panic(&["cedra", "governance", "execute-proposal", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "governance", "generate-upgrade-proposal", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "governance", "propose", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "governance", "vote", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "governance", "delegation_pool", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "governance", "delegation_pool", "vote", "--help"]).await;
    assert_cmd_not_panic(&[
        "cedra",
        "governance",
        "delegation_pool",
        "propose",
        "--help",
    ])
    .await;

    assert_cmd_not_panic(&["cedra", "info"]).await;

    assert_cmd_not_panic(&["cedra", "init", "--help"]).await;

    assert_cmd_not_panic(&["cedra", "key"]).await;
    assert_cmd_not_panic(&["cedra", "key", "generate", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "key", "extract-peer", "--help"]).await;

    assert_cmd_not_panic(&["cedra", "move"]).await;
    assert_cmd_not_panic(&["cedra", "move", "clean", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "compile", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "compile-script", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "decompile", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "disassemble", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "download", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "init", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "list", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "prove", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "publish", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "run", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "run-script", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "test", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "transactional-test", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "move", "view", "--help"]).await;

    assert_cmd_not_panic(&["cedra", "node"]).await;
    assert_cmd_not_panic(&["cedra", "node", "check-network-connectivity", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "get-stake-pool", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "analyze-validator-performance", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "bootstrap-db-from-backup", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "initialize-validator", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "join-validator-set", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "leave-validator-set", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "run-local-testnet", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "show-validator-config", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "show-validator-set", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "show-validator-stake", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "node", "update-consensus-key", "--help"]).await;
    assert_cmd_not_panic(&[
        "cedra",
        "node",
        "update-validator-network-addresses",
        "--help",
    ])
    .await;

    assert_cmd_not_panic(&["cedra", "stake"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "add-stake", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "increase-lockup", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "initialize-stake-owner", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "set-delegated-voter", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "set-operator", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "unlock-stake", "--help"]).await;
    assert_cmd_not_panic(&["cedra", "stake", "withdraw-stake", "--help"]).await;
}

/// Ensure we can parse URLs for args
#[tokio::test]
async fn ensure_can_parse_args_with_urls() {
    let result = ArgWithType::from_str("string:https://cedra.network").unwrap();
    matches!(result._ty, FunctionArgType::String);
    assert_eq!(
        result.arg,
        bcs::to_bytes(&"https://cedra.network".to_string()).unwrap()
    );
}

async fn assert_cmd_not_panic(args: &[&str]) {
    // When a command fails, it will have a panic in it due to an improperly setup command
    // thread 'main' panicked at 'Command propose: Argument names must be unique, but 'assume-yes' is
    // in use by more than one argument or group', ...

    match run_cmd(args).await {
        Ok(inner) => assert!(
            !inner.contains("panic"),
            "Failed to not panic cmd {}: {}",
            args.join(" "),
            inner
        ),
        Err(inner) => assert!(
            !inner.contains("panic"),
            "Failed to not panic cmd {}: {}",
            args.join(" "),
            inner
        ),
    }
}

async fn run_cmd(args: &[&str]) -> CliResult {
    let tool: Tool = Tool::try_parse_from(args).map_err(|msg| msg.to_string())?;
    tool.execute().await
}
