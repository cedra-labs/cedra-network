// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0
#![no_main]
#![allow(unused_imports)]

use cedra_cached_packages::cedra_stdlib;
use cedra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, SigningKey, Uniform,
};
use cedra_language_e2e_tests::{account::Account, executor::FakeExecutor};
use cedra_transaction_simulation::GENESIS_CHANGE_SET_HEAD;
use cedra_types::{
    chain_id::ChainId,
    jwks::{secure_test_rsa_jwk, AllProvidersJWKs, PatchedJWKs, ProviderJWKs},
    keyless::{
        test_utils::get_sample_iss, AnyKeylessPublicKey, Configuration, EphemeralCertificate,
        Groth16Proof, Groth16ProofAndStatement, IdCommitment, KeylessPublicKey, KeylessSignature,
        OpenIdSig, TransactionAndProof, ZeroKnowledgeSig, ZKP,
    },
    on_chain_config::OnChainConfig,
    state_store::state_key::StateKey,
    transaction::{
        authenticator::{
            AccountAuthenticator, AnyPublicKey, AnySignature, EphemeralPublicKey,
            EphemeralSignature, SingleKeyAuthenticator, TransactionAuthenticator,
        },
        ExecutionStatus, SignedTransaction, TransactionStatus,
    },
    write_set::WriteSet,
};
use cedra_vm::CedraVM;
use libfuzzer_sys::{fuzz_target, Corpus};
use move_core_types::{
    account_address::AccountAddress,
    vm_status::{StatusCode, StatusType},
};
use once_cell::sync::Lazy;
use ring::signature;
use std::sync::Arc;
mod utils;
use utils::{
    authenticator::{
        miscellaneous::{SampleJwtPayload, SAMPLE_EPK_BLINDER, SAMPLE_JWK_SK, SAMPLE_PEPPER},
        FuzzerTransactionAuthenticator, Style, TransactionState,
    },
    helpers::base64url_encode_str,
    vm::check_for_invariant_violation,
};

// genesis write set generated once for each fuzzing session
static VM: Lazy<WriteSet> = Lazy::new(|| GENESIS_CHANGE_SET_HEAD.write_set().clone());

const FUZZER_CONCURRENCY_LEVEL: usize = 1;
static TP: Lazy<Arc<rayon::ThreadPool>> = Lazy::new(|| {
    Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(FUZZER_CONCURRENCY_LEVEL)
            .build()
            .unwrap(),
    )
});

fn run_case(input: TransactionState) -> Result<(), Corpus> {
    tdbg!(&input);

    CedraVM::set_concurrency_level_once(FUZZER_CONCURRENCY_LEVEL);
    let mut vm = FakeExecutor::from_genesis_with_existing_thread_pool(
        &VM,
        ChainId::mainnet(),
        Arc::clone(&TP),
    )
    .set_not_parallel();

    let sender_acc = if true {
        // create sender pub/priv key. initialize and fund account
        vm.create_accounts(1, input.tx_auth_type.sender().fund_amount(), 0)
            .remove(0)
    } else {
        // only create sender pub/priv key. do not initialize
        Account::new()
    };

    let receiver = Account::new();

    // build tx
    let tx = sender_acc
        .transaction()
        .payload(cedra_stdlib::cedra_coin_transfer(*receiver.address(), 1))
        .sequence_number(0)
        .gas_unit_price(100)
        .max_gas_amount(1000);

    let tx_auth_type = input.tx_auth_type.clone();

    let raw_tx = tx.raw();
    let tx = match tx_auth_type {
        FuzzerTransactionAuthenticator::Ed25519 { sender: _ } => raw_tx
            .sign(&sender_acc.privkey, sender_acc.pubkey.as_ed25519().unwrap())
            .map_err(|_| Corpus::Keep)?
            .into_inner(),
        FuzzerTransactionAuthenticator::Keyless {
            sender: _,
            style,
            any_keyless_public_key,
            keyless_signature,
        } => {
            match style {
                Style::Break => {
                    // Generate a keypair for ephemeral keys
                    let private_key = Ed25519PrivateKey::generate_for_testing();
                    let public_key: Ed25519PublicKey = private_key.public_key();

                    // Create a TransactionAndProof to be signed
                    // This needs to be valid because the signature is checked in mempool (real flow)
                    let txn_and_proof = TransactionAndProof {
                        message: raw_tx.clone(),
                        proof: None,
                    };

                    // Sign the transaction
                    let signature = private_key.sign(&txn_and_proof).map_err(|_| Corpus::Keep)?;

                    // Build AnyPublicKey::Keyless
                    let any_public_key = match any_keyless_public_key {
                        AnyKeylessPublicKey::Normal(normal_key) => {
                            // TODO: think about idc, it's generated by new_from_preimage
                            AnyPublicKey::Keyless {
                                public_key: normal_key,
                            }
                        },
                        AnyKeylessPublicKey::Federated(federated_key) => {
                            // TODO: think about idc, it's generated by new_from_preimage (nested in KeylessPublicKey)
                            AnyPublicKey::FederatedKeyless {
                                public_key: federated_key,
                            }
                        },
                    };

                    // Build AnySignature::Keyless
                    let any_signature = AnySignature::Keyless {
                        signature: KeylessSignature {
                            cert: keyless_signature.cert().clone(),
                            jwt_header_json: input.tx_auth_type.get_jwt_header_json().unwrap(),
                            exp_date_secs: keyless_signature.exp_date_secs(),
                            ephemeral_pubkey: EphemeralPublicKey::ed25519(public_key),
                            ephemeral_signature: EphemeralSignature::ed25519(signature),
                        },
                    };

                    // Build an authenticator
                    let authenticator = TransactionAuthenticator::SingleSender {
                        sender: AccountAuthenticator::SingleKey {
                            authenticator: SingleKeyAuthenticator::new(
                                any_public_key,
                                any_signature,
                            ),
                        },
                    };

                    // Construct the SignedTransaction
                    SignedTransaction::new_signed_transaction(raw_tx, authenticator)
                },
                // Style::MatchJWT => {

                //     // Generate a keypair for ephemeral keys
                //     let private_key = Ed25519PrivateKey::generate_for_testing();
                //     let public_key: Ed25519PublicKey = private_key.public_key();

                //     // Create a TransactionAndProof to be signed
                //     let txn_and_proof = TransactionAndProof {
                //         message: raw_tx.clone(),
                //         proof: None,
                //     };

                //     let ks_sig: KeylessSignature;
                //     let any_public_key: AnyPublicKey;

                //     // Sign the transaction
                //     let signature = private_key.sign(&txn_and_proof).map_err(|_| Corpus::Keep)?;

                //     // Setup storage or chain state
                //     match any_keyless_public_key {
                //         AnyKeylessPublicKey::Normal(_) => {
                //             // Push JWKs via write_state_value to make them available to fetch_config
                //             let state_key = StateKey::resource(&AccountAddress::ONE, &PatchedJWKs::struct_tag()).unwrap();
                //             let iss = get_sample_iss();
                //             let patched_jwks = PatchedJWKs {
                //                 jwks: AllProvidersJWKs {
                //                     entries: vec![ProviderJWKs {
                //                         issuer: iss.into_bytes(),
                //                         version: 0,
                //                         jwks: vec![secure_test_rsa_jwk().into()],
                //                     }],
                //                 },
                //             };
                //             let data_blob = bcs::to_bytes(&patched_jwks).unwrap();
                //             vm.write_state_value(state_key, data_blob);

                //             let config = Configuration::new_for_devnet();

                //             // TODO: check if I need to move this to the top
                //             let epk = EphemeralPublicKey::ed25519(public_key);

                //             // Build JSON Web Token
                //             let mut jwt_json = <SampleJwtPayload as Default>::default();
                //             jwt_json.aud = "aud".to_string();
                //             jwt_json.sub = "sub".to_string();
                //             jwt_json.iss = get_sample_iss(); // this is contained also in JWKs issuer, need to match
                //             jwt_json.exp = keyless_signature.exp_date_secs();
                //             jwt_json.iat = keyless_signature.exp_date_secs()-10; // need to be less than exp, I should generate it to test for overflow
                //             jwt_json.nonce = OpenIdSig::reconstruct_oauth_nonce(SAMPLE_EPK_BLINDER.clone().as_slice(), keyless_signature.exp_date_secs(), &epk, &config).unwrap();

                //             let jwt_header_b64 = base64url_encode_str(&input.tx_auth_type.get_jwt_header_json().unwrap()); //SAMPLE_JWT_HEADER_B64.to_string();
                //             let jwt_payload_b64 = base64url_encode_str(&serde_json::to_string(&jwt_json).unwrap());
                //             let msg = jwt_header_b64.clone() + "." + jwt_payload_b64.as_str();
                //             let rng = ring::rand::SystemRandom::new(); //TODO: use a fixed seed
                //             let sk = *SAMPLE_JWK_SK;
                //             let mut jwt_sig = vec![0u8; sk.public_modulus_len()];

                //             sk.sign(
                //                 &signature::RSA_PKCS1_SHA256,
                //                 &rng,
                //                 msg.as_bytes(),
                //                 jwt_sig.as_mut_slice(),
                //             )
                //             .unwrap();

                //             // Build OpenIdSig
                //             let openid_sig = OpenIdSig {
                //                 jwt_sig,
                //                 jwt_payload_json: serde_json::to_string(&jwt_json).unwrap(),
                //                 uid_key: "aud".to_string(),
                //                 epk_blinder: SAMPLE_EPK_BLINDER.clone(),
                //                 pepper: SAMPLE_PEPPER.clone(),
                //                 idc_aud_val: None,
                //             };

                //             // Build KeylessSignature
                //             ks_sig = KeylessSignature {
                //                 cert: EphemeralCertificate::OpenIdSig(openid_sig.clone()), // TODO: this should be ZeroKnowledgeSig not OpenIdSig
                //                 jwt_header_json: input.tx_auth_type.get_jwt_header_json().unwrap(), //SAMPLE_JWT_HEADER_JSON.to_string(),
                //                 exp_date_secs: keyless_signature.exp_date_secs(),
                //                 ephemeral_pubkey: epk,
                //                 ephemeral_signature: EphemeralSignature::ed25519(signature),
                //             };

                //             // Build AnyPublicKey::Keyless
                //             any_public_key = AnyPublicKey::Keyless {
                //                 public_key: KeylessPublicKey {
                //                     iss_val: get_sample_iss().to_string(),
                //                     // idc here is matched against OpenIdSig::pepper, idc_aud_val, uid_key, uid_val
                //                     idc: IdCommitment::new_from_preimage(
                //                         &SAMPLE_PEPPER.clone(),
                //                         &jwt_json.aud,
                //                         "aud",
                //                         &jwt_json.sub,
                //                     )
                //                     .map_err(|_| Corpus::Keep)?,
                //                 },
                //             };

                //         },
                //         AnyKeylessPublicKey::Federated(_) => {
                //             // TODO: deploy Federated JWK at some address to be available for get_federated_jwks_onchain
                //             let acc = Account::new();
                //             let tx = acc
                //             .transaction()
                //             .gas_unit_price(100)
                //             .sequence_number(0)
                //             .payload(bcs::from_bytes(&[0, 132, 5, 161, 28, 235, 11, 6, 0, 0, 0, 7, 1, 0, 4, 2, 4, 4, 3, 8, 10, 5, 18, 36, 7, 54, 49, 8, 103, 32, 6, 135, 1, 138, 3, 0, 0, 0, 1, 1, 2, 7, 0, 1, 3, 3, 4, 0, 0, 4, 5, 2, 0, 1, 6, 12, 4, 8, 0, 8, 0, 8, 0, 8, 0, 0, 1, 10, 2, 1, 8, 0, 6, 6, 12, 10, 2, 10, 8, 0, 10, 8, 0, 10, 8, 0, 10, 8, 0, 4, 106, 119, 107, 115, 6, 115, 116, 114, 105, 110, 103, 6, 83, 116, 114, 105, 110, 103, 4, 117, 116, 102, 56, 24, 117, 112, 100, 97, 116, 101, 95, 102, 101, 100, 101, 114, 97, 116, 101, 100, 95, 106, 119, 107, 95, 115, 101, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 2, 4, 3, 82, 83, 65, 10, 2, 6, 5, 82, 83, 50, 53, 54, 10, 2, 5, 4, 65, 81, 65, 66, 10, 2, 216, 2, 214, 2, 54, 83, 55, 97, 115, 85, 117, 122, 113, 53, 81, 95, 51, 85, 57, 114, 98, 115, 45, 80, 107, 68, 86, 73, 100, 106, 103, 109, 116, 103, 87, 114, 101, 71, 53, 113, 87, 80, 115, 67, 57, 120, 88, 90, 75, 105, 77, 86, 49, 65, 105, 86, 57, 76, 88, 121, 113, 81, 115, 65, 89, 112, 67, 113, 69, 68, 77, 51, 88, 98, 102, 109, 90, 113, 71, 98, 52, 56, 121, 76, 104, 98, 95, 88, 113, 90, 97, 75, 103, 83, 89, 97, 67, 95, 104, 50, 68, 106, 77, 55, 108, 103, 114, 73, 81, 65, 112, 57, 57, 48, 50, 82, 114, 56, 102, 85, 109, 76, 78, 50, 105, 118, 114, 53, 116, 110, 76, 120, 85, 85, 79, 110, 77, 79, 99, 50, 83, 81, 116, 114, 57, 100, 103, 122, 84, 79, 78, 89, 87, 53, 90, 117, 51, 80, 119, 121, 118, 65, 87, 107, 53, 68, 54, 117, 101, 73, 85, 104, 76, 116, 89, 122, 112, 99, 66, 45, 101, 116, 111, 78, 100, 76, 51, 73, 114, 50, 55, 52, 54, 75, 73, 121, 95, 86, 85, 115, 68, 119, 65, 77, 55, 100, 104, 114, 113, 83, 75, 56, 85, 50, 120, 70, 67, 71, 108, 97, 117, 52, 105, 107, 79, 84, 116, 118, 122, 68, 111, 119, 110, 65, 77, 72, 77, 114, 102, 69, 55, 113, 49, 66, 54, 87, 90, 81, 68, 65, 81, 108, 66, 109, 120, 82, 81, 115, 121, 75, 108, 110, 53, 68, 73, 115, 75, 118, 54, 120, 97, 117, 78, 115, 72, 82, 103, 66, 65, 75, 99, 116, 85, 120, 90, 71, 56, 77, 52, 81, 74, 73, 120, 51, 83, 54, 65, 117, 103, 104, 100, 51, 82, 90, 67, 52, 67, 97, 53, 65, 101, 57, 102, 100, 56, 76, 56, 109, 108, 78, 89, 66, 67, 114, 81, 104, 79, 90, 55, 100, 83, 48, 102, 52, 97, 116, 52, 97, 114, 108, 76, 99, 97, 106, 116, 119, 10, 2, 19, 18, 116, 101, 115, 116, 46, 111, 105, 100, 99, 46, 112, 114, 111, 118, 105, 100, 101, 114, 0, 0, 1, 24, 7, 0, 17, 0, 12, 3, 7, 1, 17, 0, 12, 1, 7, 2, 17, 0, 12, 2, 7, 3, 17, 0, 12, 4, 11, 0, 7, 4, 11, 3, 64, 4, 1, 0, 0, 0, 0, 0, 0, 0, 11, 1, 64, 4, 1, 0, 0, 0, 0, 0, 0, 0, 11, 2, 64, 4, 1, 0, 0, 0, 0, 0, 0, 0, 11, 4, 64, 4, 1, 0, 0, 0, 0, 0, 0, 0, 17, 1, 2, 0, 0]).unwrap())
                //             .sign();

                //             vm.execute_and_apply(tx.clone());

                //             let public_inputs_hash = fr_to_bytes_le(&get_public_inputs_hash(&sig, pk, jwk, config).unwrap());

                //             // Build ZKP
                //             let proof = ZKP::Groth16(
                //                 Groth16Proof {
                //                     a: sig.a,
                //                     b: sig.b,
                //                     c: sig.c,
                //                 }
                //             );

                //             // Build ZeroKnowledgeSig
                //             let zk_sig = ZeroKnowledgeSig {
                //                 proof,
                //                 exp_horizon_secs: 0,
                //                 extra_field: None,
                //                 override_aud_val: None,
                //                 training_wheels_signature: None,
                //             };

                //             // Build KeylessSignature
                //             ks_sig = KeylessSignature {
                //                 cert: EphemeralCertificate::ZeroKnowledgeSig(zk_sig.clone()),
                //                 jwt_header_json: input.tx_auth_type.get_jwt_header_json().unwrap(), //SAMPLE_JWT_HEADER_JSON.to_string(),
                //                 exp_date_secs: keyless_signature.exp_date_secs(),
                //                 ephemeral_pubkey: epk,
                //                 ephemeral_signature: EphemeralSignature::ed25519(signature),
                //             };

                //             let fed_pk = FederatedKeylessPublicKey {
                //                 acc,
                //                 pk: SAMPLE_PK.clone(),
                //             };
                //         },
                //     };

                //     // Build AnySignature::Keyless
                //     let any_signature = AnySignature::Keyless {
                //         signature: ks_sig,
                //     };

                //     // Build an authenticator
                //     let authenticator = TransactionAuthenticator::SingleSender {
                //         sender: AccountAuthenticator::SingleKey {
                //             authenticator: SingleKeyAuthenticator::new(any_public_key, any_signature),
                //         },
                //     };

                //     // Construct the SignedTransaction
                //     SignedTransaction::new_signed_transaction(raw_tx, authenticator)
                // },
                /*
                Style::MatchKeys => {
                    // Generate a keypair for ephemeral keys
                    let private_key = Ed25519PrivateKey::generate_for_testing();
                    let public_key: Ed25519PublicKey = private_key.public_key();

                    // Create a TransactionAndProof to be signed
                    let txn_and_proof = TransactionAndProof {
                        message: raw_tx.clone(),
                        proof: None,
                    };

                    // Sign the transaction
                    let signature = private_key.sign(&txn_and_proof).map_err(|_| Corpus::Keep)?;

                    // Build AnyPublicKey::Keyless
                    let any_public_key = AnyPublicKey::Keyless {
                        public_key: KeylessPublicKey {
                            iss_val: "test.oidc.provider".to_string(),
                            idc: IdCommitment::new_from_preimage(
                                &Pepper::from_number(0x5678),
                                "aud",
                                "uid_key",
                                "uid_val",
                            )
                            .map_err(|_| Corpus::Keep)?,
                        },
                    };

                    /*
                    EphemeralCertificate::OpenIdSig(OpenIdSig {
                                jwt_sig: vec![],
                                jwt_payload_json: "jwt_payload_json".to_string(),
                                uid_key: "uid_key".to_string(),
                                epk_blinder: b"epk_blinder".to_vec(),
                                pepper: Pepper::from_number(0x1234),
                                idc_aud_val: None,
                            })
                    */

                    // Build AnySignature::Keyless
                    let any_signature = AnySignature::Keyless {
                        signature: KeylessSignature {
                            cert: keyless_signature.cert().clone(),
                            jwt_header_json: input.tx_auth_type.get_jwt_header_json().unwrap(),
                            exp_date_secs: keyless_signature.exp_date_secs(),
                            ephemeral_pubkey: EphemeralPublicKey::ed25519(public_key),
                            ephemeral_signature: EphemeralSignature::ed25519(signature),
                        },
                    };

                    // Build an authenticator
                    let authenticator = TransactionAuthenticator::SingleSender {
                        sender: AccountAuthenticator::SingleKey {
                            authenticator: SingleKeyAuthenticator::new(any_public_key, any_signature),
                        },
                    };

                    // Construct the SignedTransaction
                    SignedTransaction::new_signed_transaction(raw_tx, authenticator)
                }
                */
            }
        },
        FuzzerTransactionAuthenticator::MultiAgent {
            sender: _,
            secondary_signers,
        } => {
            // higher number here slows down fuzzer significatly due to slow signing process.
            if secondary_signers.len() > 10 {
                return Err(Corpus::Keep);
            }
            let secondary_accs: Vec<_> = secondary_signers
                .iter()
                .map(|acc| acc.convert_account(&mut vm))
                .collect();
            let secondary_signers = secondary_accs.iter().map(|acc| *acc.address()).collect();
            let secondary_private_keys = secondary_accs.iter().map(|acc| &acc.privkey).collect();
            raw_tx
                .sign_multi_agent(
                    &sender_acc.privkey,
                    secondary_signers,
                    secondary_private_keys,
                )
                .map_err(|_| Corpus::Keep)?
                .into_inner()
        },
        FuzzerTransactionAuthenticator::FeePayer {
            sender: _,
            secondary_signers,
            fee_payer,
        } => {
            // higher number here slows down fuzzer significatly due to slow signing process.
            if secondary_signers.len() > 10 {
                return Err(Corpus::Keep);
            }
            let secondary_accs: Vec<_> = secondary_signers
                .iter()
                .map(|acc| acc.convert_account(&mut vm))
                .collect();

            let secondary_signers = secondary_accs.iter().map(|acc| *acc.address()).collect();
            let secondary_private_keys = secondary_accs.iter().map(|acc| &acc.privkey).collect();
            let fee_payer_acc = fee_payer.convert_account(&mut vm);
            raw_tx
                .sign_fee_payer(
                    &sender_acc.privkey,
                    secondary_signers,
                    secondary_private_keys,
                    *fee_payer_acc.address(),
                    &fee_payer_acc.privkey,
                )
                .map_err(|_| Corpus::Keep)?
                .into_inner()
        },
    };

    // exec tx
    tdbg!("exec start");

    let res = vm.execute_block(vec![tx.clone()]);

    let res = res
        .map_err(|e| {
            check_for_invariant_violation(e);
            Corpus::Keep
        })?
        .pop()
        .expect("expect 1 output");
    tdbg!("exec end");

    // if error exit gracefully
    let status = match tdbg!(res.status()) {
        TransactionStatus::Keep(status) => status,
        TransactionStatus::Discard(e) => {
            if e.status_type() == StatusType::InvariantViolation {
                panic!("invariant violation {:?}", e);
            }
            return Err(Corpus::Keep);
        },
        _ => return Err(Corpus::Keep),
    };
    match tdbg!(status) {
        ExecutionStatus::Success => (),
        ExecutionStatus::MiscellaneousError(e) => {
            if let Some(e) = e {
                if e.status_type() == StatusType::InvariantViolation
                    && *e != StatusCode::TYPE_RESOLUTION_FAILURE
                    && *e != StatusCode::STORAGE_ERROR
                {
                    panic!("invariant violation {:?}", e);
                }
            }
            return Err(Corpus::Keep);
        },
        _ => return Err(Corpus::Keep),
    };

    Ok(())
}

fuzz_target!(|fuzz_data: TransactionState| -> Corpus {
    run_case(fuzz_data).err().unwrap_or(Corpus::Keep)
});
