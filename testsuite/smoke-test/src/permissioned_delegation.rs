// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SwarmBuilder;
use cedra::move_tool::MemberId;
use cedra_cached_packages::cedra_stdlib;
use cedra_crypto::SigningKey;
use cedra_forge::Swarm;
use cedra_types::function_info::FunctionInfo;
use move_core_types::account_address::AccountAddress;
use std::{str::FromStr, sync::Arc};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_permissioned_delegation() {
    let (swarm, mut cli, _faucet) = SwarmBuilder::new_local(1)
        .with_cedra()
        .build_with_cli(0)
        .await;
    let mut info = swarm.cedra_public_info();

    let mut account1 = info
        .create_and_fund_user_account(100_000_000_000)
        .await
        .unwrap();
    let account2 = info.random_account();
    info.create_user_account(account2.public_key())
        .await
        .unwrap();
    let account2_private_key = account2.private_key().clone();
    let account2_public_key = account2.public_key().clone();
    let idx = cli.add_account_to_cli(account1.private_key().clone());

    assert_eq!(
        Some(true),
        cli.run_function(
            idx,
            None,
            MemberId::from_str("0x1::account_abstraction::add_authentication_function").unwrap(),
            vec![
                "address:0x1",
                "string:permissioned_delegation",
                "string:authenticate"
            ],
            vec![]
        )
        .await
        .unwrap()
        .success
    );
    account1.increment_sequence_number();

    // Setup permissions: 10 Cedra allowance, and 0.1 Cedra gas.
    let script = format!(
        r#"
    script {{
    use cedra_std::ed25519;
    use cedra_framework::coin;
    use cedra_framework::permissioned_delegation;
    use cedra_framework::primary_fungible_store;
    use cedra_framework::transaction_validation;
    fun main(sender: &signer) {{
        coin::migrate_to_fungible_store<cedra_framework::cedra_coin::CedraCoin>(sender);
        let key = permissioned_delegation::gen_ed25519_key(ed25519::new_unvalidated_public_key_from_bytes(x"{}"));
        let permissioned_signer = permissioned_delegation::add_permissioned_handle(sender, key, std::option::none(), {});
        primary_fungible_store::grant_apt_permission(sender, &permissioned_signer, 1000000000); // 10 apt
        transaction_validation::grant_gas_permission(sender, &permissioned_signer, 100000000); // 1 apt because that is the max_gas
    }}
    }}
    "#,
        hex::encode(account2_public_key.to_bytes()),
        u64::MAX,
    );
    assert_eq!(
        Some(true),
        cli.run_script(idx, &script).await.unwrap().success
    );
    account1.increment_sequence_number();

    let func_info = FunctionInfo::new(
        AccountAddress::ONE,
        "permissioned_delegation".to_string(),
        "authenticate".to_string(),
    );
    account1.set_abstraction_auth(
        func_info,
        Arc::new(move |x: &[u8]| {
            let mut authenticator = vec![];
            authenticator.extend(bcs::to_bytes(&account2_public_key.to_bytes().to_vec()).unwrap());
            authenticator.extend(
                bcs::to_bytes(
                    &account2_private_key
                        .sign_arbitrary_message(x)
                        .to_bytes()
                        .to_vec(),
                )
                .unwrap(),
            );
            authenticator
        }),
    );

    // Transfer 1 Cedra and 2 Cedra.
    let transfer_txn = account1.sign_aa_transaction_with_transaction_builder(
        vec![],
        None,
        info.transaction_factory()
            .payload(cedra_stdlib::cedra_account_fungible_transfer_only(
                account2.address(),
                100000000,
            )),
    );
    info.client().submit_and_wait(&transfer_txn).await.unwrap();

    // gas permission check failed.
    let transfer_txn = account1.sign_aa_transaction_with_transaction_builder(
        vec![],
        None,
        info.transaction_factory()
            .payload(cedra_stdlib::cedra_account_fungible_transfer_only(
                account2.address(),
                200000000,
            )),
    );
    assert!(info.client().submit_and_wait(&transfer_txn).await.is_err());
    account1.decrement_sequence_number();

    let transfer_txn = account1.sign_aa_transaction_with_transaction_builder(
        vec![],
        None,
        info.transaction_factory()
            .payload(cedra_stdlib::cedra_account_fungible_transfer_only(
                account2.address(),
                200000000,
            ))
            .max_gas_amount(50000000),
    );
    info.client().submit_and_wait(&transfer_txn).await.unwrap();

    let transfer_txn = account1.sign_aa_transaction_with_transaction_builder(
        vec![],
        None,
        info.transaction_factory()
            .payload(cedra_stdlib::cedra_account_fungible_transfer_only(
                account2.address(),
                700000001,
            ))
            .max_gas_amount(50000000),
    );
    assert!(info.client().submit_and_wait(&transfer_txn).await.is_err());

    let transfer_txn = account1.sign_aa_transaction_with_transaction_builder(
        vec![],
        None,
        info.transaction_factory()
            .payload(cedra_stdlib::cedra_account_fungible_transfer_only(
                account2.address(),
                700000000,
            ))
            .max_gas_amount(50000000),
    );
    info.client().submit_and_wait(&transfer_txn).await.unwrap();
}
