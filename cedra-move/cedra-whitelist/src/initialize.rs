// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use cedra_sdk::{
    coin_client::CoinClient,
    crypto::{ed25519::Ed25519PrivateKey, Uniform},
    move_types::{identifier::Identifier, language_storage::ModuleId},
    rest_client::{Client, FaucetClient},
    transaction_builder::TransactionBuilder,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, TransactionPayload},
        LocalAccount,
    },
};
use hex::FromHex;
use move_vm_runtime::move_vm::MoveVM;
use once_cell::sync::Lazy;
use std::str::FromStr;
use url::Url;

use std::time::{SystemTime, UNIX_EPOCH};
// :!:>section_1c
static NODE_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("CEDRA_NODE_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("http://127.0.0.1:8080"),
    )
    .unwrap()
});

static FAUCET_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("CEDRA_FAUCET_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("http://127.0.0.1:8011"),
    )
    .unwrap()
});
// <:!:section_1c
const FRAMEWORK_ADDRESS: &str = "0x1";

pub async fn initialize() -> Result<()> {
    let rest_client = Client::new(NODE_URL.clone());
    let faucet_client = FaucetClient::new(FAUCET_URL.clone(), NODE_URL.clone());
    let coin_client = CoinClient::new(&rest_client);
    let owner_address = AccountAddress::from_hex_literal(
        "0xc7d8de66f1304a54a337fe5b91113dfcb702d35551a1a5c76a50eb9d2fe14c3c",
    )?;

    let account_data = rest_client
        .get_account(owner_address)
        .await
        .context("Failed to get account info")?;
    // let seq_number = account_data.seq_number;
    // dbg!(&seq_number);
    let hex_str = "accbc0397cc72295c533aa08fe3eaa178d79308558a58d36bba77c5803399d8d"; // no 0x prefix
    let key_bytes = <[u8; 32]>::from_hex(hex_str)
        .context("Invalid hex or wrong length for Ed25519 private key")?;

    let mut alice = LocalAccount::generate(&mut rand::rngs::OsRng);

    faucet_client
        .fund(alice.address(), 500_000_000)
        .await
        .context("Failed to fund Alice's account")?;
    println!("\n=== Initial Balances ===");
    println!(
        "Alice: {:?}",
        coin_client
            .get_account_balance(&alice.address())
            .await
            .context("Failed to get Alice's account balance")?
    );

    let private_key = Ed25519PrivateKey::try_from(&key_bytes[..])?;
    let mut owner = LocalAccount::new(owner_address, private_key, 0);

    faucet_client
        .fund(owner.address(), 500_000_000)
        .await
        .context("Failed to fund owner's account")?;

    println!("\n=== Initial Balances ===");
    println!(
        "Owner: {:?}",
        coin_client
            .get_account_balance(&owner.address())
            .await
            .context("Failed to get Owner's account balance")?
    );
    let payload = TransactionBuilder::new(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::ONE, Identifier::new("whitelist")?),
            Identifier::new("initialize_whitelist")?,
            vec![], // No type arguments
            vec![], // No function arguments
        )),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()d
            .as_secs()
            + 10,
        ChainId::new(2),
    );

    // 4. Sign and submit transaction
    let txn = alice.sign_with_transaction_builder(payload);

    rest_client
        .submit_and_wait(&txn)
        .await
        .context("Failed to initialize whitelist")?;

    println!("Whitelist initialized successfully!");

    Ok(())
}
