// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{components::get_signer_arg, utils::*};
use anyhow::Result;
use cedra_crypto::HashValue;
use cedra_types::on_chain_config::{FeatureFlag as CedraFeatureFlag, Features as CedraFeatures};
use move_model::{code_writer::CodeWriter, emit, emitln, model::Loc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Deserialize, PartialEq, Eq, Serialize, Debug)]
pub struct Features {
    #[serde(default)]
    pub enabled: Vec<FeatureFlag>,
    #[serde(default)]
    pub disabled: Vec<FeatureFlag>,
}

impl Features {
    pub fn empty() -> Self {
        Self {
            enabled: vec![],
            disabled: vec![],
        }
    }

    pub fn squash(&mut self, rhs: Self) {
        let mut enabled: HashSet<_> = self.enabled.iter().cloned().collect();
        let mut disabled: HashSet<_> = self.disabled.iter().cloned().collect();
        let to_enable: HashSet<_> = rhs.enabled.into_iter().collect();
        let to_disable: HashSet<_> = rhs.disabled.into_iter().collect();

        disabled = disabled.difference(&to_enable).cloned().collect();
        enabled.extend(to_enable);

        enabled = enabled.difference(&to_disable).cloned().collect();
        disabled.extend(to_disable);

        self.enabled = enabled.into_iter().collect();
        self.disabled = disabled.into_iter().collect();
    }

    pub fn is_empty(&self) -> bool {
        self.enabled.is_empty() && self.disabled.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, EnumIter, PartialEq, Eq, Serialize, Hash)]
#[allow(non_camel_case_types)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlag {
    CodeDependencyCheck,
    CollectAndDistributeGasFees,
    TreatFriendAsPrivate,
    Sha512AndRipeMd160Natives,
    CedraStdChainIdNatives,
    VMBinaryFormatV6,
    MultiEd25519PkValidateV2Natives,
    Blake2b256Native,
    ResourceGroups,
    MultisigAccounts,
    DelegationPools,
    CryptographyAlgebraNatives,
    Bls12381Structures,
    Ed25519PubkeyValidateReturnFalseWrongLength,
    StructConstructors,
    PeriodicalRewardRateReduction,
    PartialGovernanceVoting,
    SignatureCheckerV2,
    StorageSlotMetadata,
    ChargeInvariantViolation,
    DelegationPoolPartialGovernanceVoting,
    GasPayerEnabled,
    CedraUniqueIdentifiers,
    BulletproofsNatives,
    SignerNativeFormatFix,
    ModuleEvent,
    EmitFeeStatement,
    StorageDeletionRefund,
    AggregatorV2Api,
    SignatureCheckerV2ScriptFix,
    SaferResourceGroups,
    SaferMetadata,
    SingleSenderAuthenticator,
    SponsoredAutomaticAccountCreation,
    FeePayerAccountOptional,
    AggregatorV2DelayedFields,
    ConcurrentTokenV2,
    LimitMaxIdentifierLength,
    OperatorBeneficiaryChange,
    VMBinaryFormatV7,
    ResourceGroupsSplitInVmChangeSet,
    CommissionChangeDelegationPool,
    Bn254Structures,
    WebAuthnSignature,
    ReconfigureWithDkg,
    KeylessAccounts,
    KeylessButZklessAccounts,
    RemoveDetailedError,
    JwkConsensus,
    ConcurrentFungibleAssets,
    RefundableBytes,
    ObjectCodeDeployment,
    MaxObjectNestingCheck,
    KeylessAccountsWithPasskeys,
    MultisigV2Enhancement,
    DelegationPoolAllowlisting,
    ModuleEventMigration,
    RejectUnstableBytecode,
    TransactionContextExtension,
    CoinToFungibleAssetMigration,
    PrimaryCedraFungibleStoreAtUserAddress,
    ObjectNativeDerivedAddress,
    DispatchableFungibleAsset,
    NewAccountsDefaultToFaAptStore,
    OperationsDefaultToFaAptStore,
    AggregatorV2IsAtLeastApi,
    ConcurrentFungibleBalance,
    DefaultToConcurrentFungibleBalance,
    LimitVMTypeSize,
    AbortIfMultisigPayloadMismatch,
    DisallowUserNative,
    AllowSerializedScriptArgs,
    UseCompatibilityCheckerV2,
    EnableEnumTypes,
    EnableResourceAccessControl,
    RejectUnstableBytecodeForScript,
    FederatedKeyless,
    TransactionSimulationEnhancement,
    CollectionOwner,
    NativeMemoryOperations,
    EnableLoaderV2,
    DisallowInitModuleToPublishModules,
    EnableCallTreeAndInstructionVMCache,
    PermissionedSigner,
    AccountAbstraction,
    VMBinaryFormatV8,
    BulletproofsBatchNatives,
    DerivableAccountAbstraction,
    EnableFunctionValues,
    NewAccountsDefaultToFaStore,
    DefaultAccountResource,
    JwkConsensusPerKeyMode,
    TransactionPayloadV2,
    OrderlessTransactions,
    FeeV2,
}

fn generate_features_blob(writer: &CodeWriter, data: &[u64]) {
    emitln!(writer, "vector[");
    writer.indent();
    for (i, b) in data.iter().enumerate() {
        if i % 20 == 0 {
            if i > 0 {
                emitln!(writer);
            }
        } else {
            emit!(writer, " ");
        }
        emit!(writer, "{},", b);
    }
    emitln!(writer);
    writer.unindent();
    emit!(writer, "]")
}

pub fn generate_feature_upgrade_proposal(
    features: &Features,
    is_testnet: bool,
    next_execution_hash: Option<HashValue>,
    is_multi_step: bool,
) -> Result<Vec<(String, String)>> {
    let signer_arg = get_signer_arg(is_testnet, &next_execution_hash);
    let mut result = vec![];

    let enabled = features
        .enabled
        .iter()
        .map(|f| CedraFeatureFlag::from(f.clone()) as u64)
        .collect::<Vec<_>>();
    let disabled = features
        .disabled
        .iter()
        .map(|f| CedraFeatureFlag::from(f.clone()) as u64)
        .collect::<Vec<_>>();

    assert!(enabled.len() < u16::MAX as usize);
    assert!(disabled.len() < u16::MAX as usize);

    let writer = CodeWriter::new(Loc::default());

    emitln!(writer, "// Modifying on-chain feature flags: ");
    emitln!(writer, "// Enabled Features: {:?}", features.enabled);
    emitln!(writer, "// Disabled Features: {:?}", features.disabled);
    emitln!(writer, "//");

    let proposal = generate_governance_proposal(
        &writer,
        is_testnet,
        next_execution_hash,
        is_multi_step,
        &["std::features"],
        |writer| {
            emit!(writer, "let enabled_blob: vector<u64> = ");
            generate_features_blob(writer, &enabled);
            emitln!(writer, ";\n");

            emit!(writer, "let disabled_blob: vector<u64> = ");
            generate_features_blob(writer, &disabled);
            emitln!(writer, ";\n");

            emitln!(
                writer,
                "features::change_feature_flags_for_next_epoch({}, enabled_blob, disabled_blob);",
                signer_arg
            );
            emitln!(writer, "cedra_governance::reconfigure({});", signer_arg);
        },
    );

    result.push(("features".to_string(), proposal));
    Ok(result)
}

impl From<FeatureFlag> for CedraFeatureFlag {
    fn from(f: FeatureFlag) -> Self {
        match f {
            FeatureFlag::CodeDependencyCheck => CedraFeatureFlag::CODE_DEPENDENCY_CHECK,
            FeatureFlag::CollectAndDistributeGasFees => {
                CedraFeatureFlag::_DEPRECATED_COLLECT_AND_DISTRIBUTE_GAS_FEES
            },
            FeatureFlag::TreatFriendAsPrivate => CedraFeatureFlag::TREAT_FRIEND_AS_PRIVATE,
            FeatureFlag::Sha512AndRipeMd160Natives => {
                CedraFeatureFlag::SHA_512_AND_RIPEMD_160_NATIVES
            },
            FeatureFlag::CedraStdChainIdNatives => CedraFeatureFlag::CEDRA_STD_CHAIN_ID_NATIVES,
            FeatureFlag::VMBinaryFormatV6 => CedraFeatureFlag::VM_BINARY_FORMAT_V6,
            FeatureFlag::VMBinaryFormatV7 => CedraFeatureFlag::VM_BINARY_FORMAT_V7,
            FeatureFlag::VMBinaryFormatV8 => CedraFeatureFlag::VM_BINARY_FORMAT_V8,
            FeatureFlag::MultiEd25519PkValidateV2Natives => {
                CedraFeatureFlag::MULTI_ED25519_PK_VALIDATE_V2_NATIVES
            },
            FeatureFlag::Blake2b256Native => CedraFeatureFlag::BLAKE2B_256_NATIVE,
            FeatureFlag::ResourceGroups => CedraFeatureFlag::RESOURCE_GROUPS,
            FeatureFlag::MultisigAccounts => CedraFeatureFlag::MULTISIG_ACCOUNTS,
            FeatureFlag::DelegationPools => CedraFeatureFlag::DELEGATION_POOLS,
            FeatureFlag::CryptographyAlgebraNatives => {
                CedraFeatureFlag::CRYPTOGRAPHY_ALGEBRA_NATIVES
            },
            FeatureFlag::Bls12381Structures => CedraFeatureFlag::BLS12_381_STRUCTURES,
            FeatureFlag::Ed25519PubkeyValidateReturnFalseWrongLength => {
                CedraFeatureFlag::ED25519_PUBKEY_VALIDATE_RETURN_FALSE_WRONG_LENGTH
            },
            FeatureFlag::StructConstructors => CedraFeatureFlag::STRUCT_CONSTRUCTORS,
            FeatureFlag::PeriodicalRewardRateReduction => {
                CedraFeatureFlag::PERIODICAL_REWARD_RATE_DECREASE
            },
            FeatureFlag::PartialGovernanceVoting => CedraFeatureFlag::PARTIAL_GOVERNANCE_VOTING,
            FeatureFlag::SignatureCheckerV2 => CedraFeatureFlag::SIGNATURE_CHECKER_V2,
            FeatureFlag::StorageSlotMetadata => CedraFeatureFlag::STORAGE_SLOT_METADATA,
            FeatureFlag::ChargeInvariantViolation => CedraFeatureFlag::CHARGE_INVARIANT_VIOLATION,
            FeatureFlag::DelegationPoolPartialGovernanceVoting => {
                CedraFeatureFlag::DELEGATION_POOL_PARTIAL_GOVERNANCE_VOTING
            },
            FeatureFlag::GasPayerEnabled => CedraFeatureFlag::GAS_PAYER_ENABLED,
            FeatureFlag::CedraUniqueIdentifiers => CedraFeatureFlag::CEDRA_UNIQUE_IDENTIFIERS,
            FeatureFlag::BulletproofsNatives => CedraFeatureFlag::BULLETPROOFS_NATIVES,
            FeatureFlag::SignerNativeFormatFix => CedraFeatureFlag::SIGNER_NATIVE_FORMAT_FIX,
            FeatureFlag::ModuleEvent => CedraFeatureFlag::MODULE_EVENT,
            FeatureFlag::EmitFeeStatement => CedraFeatureFlag::EMIT_FEE_STATEMENT,
            FeatureFlag::StorageDeletionRefund => CedraFeatureFlag::STORAGE_DELETION_REFUND,
            FeatureFlag::AggregatorV2Api => CedraFeatureFlag::AGGREGATOR_V2_API,
            FeatureFlag::SignatureCheckerV2ScriptFix => {
                CedraFeatureFlag::SIGNATURE_CHECKER_V2_SCRIPT_FIX
            },
            FeatureFlag::SaferResourceGroups => CedraFeatureFlag::SAFER_RESOURCE_GROUPS,
            FeatureFlag::SaferMetadata => CedraFeatureFlag::SAFER_METADATA,
            FeatureFlag::SingleSenderAuthenticator => CedraFeatureFlag::SINGLE_SENDER_AUTHENTICATOR,
            FeatureFlag::SponsoredAutomaticAccountCreation => {
                CedraFeatureFlag::SPONSORED_AUTOMATIC_ACCOUNT_V1_CREATION
            },
            FeatureFlag::FeePayerAccountOptional => CedraFeatureFlag::FEE_PAYER_ACCOUNT_OPTIONAL,
            FeatureFlag::AggregatorV2DelayedFields => {
                CedraFeatureFlag::AGGREGATOR_V2_DELAYED_FIELDS
            },
            FeatureFlag::ConcurrentTokenV2 => CedraFeatureFlag::CONCURRENT_TOKEN_V2,
            FeatureFlag::LimitMaxIdentifierLength => CedraFeatureFlag::LIMIT_MAX_IDENTIFIER_LENGTH,
            FeatureFlag::OperatorBeneficiaryChange => CedraFeatureFlag::OPERATOR_BENEFICIARY_CHANGE,
            FeatureFlag::ResourceGroupsSplitInVmChangeSet => {
                CedraFeatureFlag::RESOURCE_GROUPS_SPLIT_IN_VM_CHANGE_SET
            },
            FeatureFlag::CommissionChangeDelegationPool => {
                CedraFeatureFlag::COMMISSION_CHANGE_DELEGATION_POOL
            },
            FeatureFlag::Bn254Structures => CedraFeatureFlag::BN254_STRUCTURES,
            FeatureFlag::WebAuthnSignature => CedraFeatureFlag::WEBAUTHN_SIGNATURE,
            FeatureFlag::ReconfigureWithDkg => CedraFeatureFlag::_DEPRECATED_RECONFIGURE_WITH_DKG,
            FeatureFlag::KeylessAccounts => CedraFeatureFlag::KEYLESS_ACCOUNTS,
            FeatureFlag::KeylessButZklessAccounts => CedraFeatureFlag::KEYLESS_BUT_ZKLESS_ACCOUNTS,
            FeatureFlag::RemoveDetailedError => {
                CedraFeatureFlag::_DEPRECATED_REMOVE_DETAILED_ERROR_FROM_HASH
            },
            FeatureFlag::JwkConsensus => CedraFeatureFlag::JWK_CONSENSUS,
            FeatureFlag::ConcurrentFungibleAssets => CedraFeatureFlag::CONCURRENT_FUNGIBLE_ASSETS,
            FeatureFlag::RefundableBytes => CedraFeatureFlag::REFUNDABLE_BYTES,
            FeatureFlag::ObjectCodeDeployment => CedraFeatureFlag::OBJECT_CODE_DEPLOYMENT,
            FeatureFlag::MaxObjectNestingCheck => CedraFeatureFlag::MAX_OBJECT_NESTING_CHECK,
            FeatureFlag::KeylessAccountsWithPasskeys => {
                CedraFeatureFlag::KEYLESS_ACCOUNTS_WITH_PASSKEYS
            },
            FeatureFlag::MultisigV2Enhancement => CedraFeatureFlag::MULTISIG_V2_ENHANCEMENT,
            FeatureFlag::DelegationPoolAllowlisting => {
                CedraFeatureFlag::DELEGATION_POOL_ALLOWLISTING
            },
            FeatureFlag::ModuleEventMigration => CedraFeatureFlag::MODULE_EVENT_MIGRATION,
            FeatureFlag::RejectUnstableBytecode => CedraFeatureFlag::_REJECT_UNSTABLE_BYTECODE,
            FeatureFlag::TransactionContextExtension => {
                CedraFeatureFlag::TRANSACTION_CONTEXT_EXTENSION
            },
            FeatureFlag::CoinToFungibleAssetMigration => {
                CedraFeatureFlag::COIN_TO_FUNGIBLE_ASSET_MIGRATION
            },
            FeatureFlag::PrimaryCedraFungibleStoreAtUserAddress => {
                CedraFeatureFlag::PRIMARY_CEDRA_FUNGIBLE_STORE_AT_USER_ADDRESS
            },
            FeatureFlag::ObjectNativeDerivedAddress => {
                CedraFeatureFlag::OBJECT_NATIVE_DERIVED_ADDRESS
            },
            FeatureFlag::DispatchableFungibleAsset => CedraFeatureFlag::DISPATCHABLE_FUNGIBLE_ASSET,
            FeatureFlag::NewAccountsDefaultToFaAptStore => {
                CedraFeatureFlag::NEW_ACCOUNTS_DEFAULT_TO_FA_CEDRA_STORE
            },
            FeatureFlag::OperationsDefaultToFaAptStore => {
                CedraFeatureFlag::OPERATIONS_DEFAULT_TO_FA_CEDRA_STORE
            },
            FeatureFlag::AggregatorV2IsAtLeastApi => {
                CedraFeatureFlag::AGGREGATOR_V2_IS_AT_LEAST_API
            },
            FeatureFlag::ConcurrentFungibleBalance => CedraFeatureFlag::CONCURRENT_FUNGIBLE_BALANCE,
            FeatureFlag::DefaultToConcurrentFungibleBalance => {
                CedraFeatureFlag::DEFAULT_TO_CONCURRENT_FUNGIBLE_BALANCE
            },
            FeatureFlag::LimitVMTypeSize => CedraFeatureFlag::_LIMIT_VM_TYPE_SIZE,
            FeatureFlag::AbortIfMultisigPayloadMismatch => {
                CedraFeatureFlag::ABORT_IF_MULTISIG_PAYLOAD_MISMATCH
            },
            FeatureFlag::DisallowUserNative => CedraFeatureFlag::_DISALLOW_USER_NATIVES,
            FeatureFlag::AllowSerializedScriptArgs => {
                CedraFeatureFlag::ALLOW_SERIALIZED_SCRIPT_ARGS
            },
            FeatureFlag::UseCompatibilityCheckerV2 => {
                CedraFeatureFlag::_USE_COMPATIBILITY_CHECKER_V2
            },
            FeatureFlag::EnableEnumTypes => CedraFeatureFlag::ENABLE_ENUM_TYPES,
            FeatureFlag::EnableResourceAccessControl => {
                CedraFeatureFlag::ENABLE_RESOURCE_ACCESS_CONTROL
            },
            FeatureFlag::RejectUnstableBytecodeForScript => {
                CedraFeatureFlag::_REJECT_UNSTABLE_BYTECODE_FOR_SCRIPT
            },
            FeatureFlag::FederatedKeyless => CedraFeatureFlag::FEDERATED_KEYLESS,
            FeatureFlag::TransactionSimulationEnhancement => {
                CedraFeatureFlag::TRANSACTION_SIMULATION_ENHANCEMENT
            },
            FeatureFlag::CollectionOwner => CedraFeatureFlag::COLLECTION_OWNER,
            FeatureFlag::NativeMemoryOperations => CedraFeatureFlag::NATIVE_MEMORY_OPERATIONS,
            FeatureFlag::EnableLoaderV2 => CedraFeatureFlag::_ENABLE_LOADER_V2,
            FeatureFlag::DisallowInitModuleToPublishModules => {
                CedraFeatureFlag::_DISALLOW_INIT_MODULE_TO_PUBLISH_MODULES
            },
            FeatureFlag::EnableCallTreeAndInstructionVMCache => {
                CedraFeatureFlag::ENABLE_CALL_TREE_AND_INSTRUCTION_VM_CACHE
            },
            FeatureFlag::PermissionedSigner => CedraFeatureFlag::PERMISSIONED_SIGNER,
            FeatureFlag::AccountAbstraction => CedraFeatureFlag::ACCOUNT_ABSTRACTION,
            FeatureFlag::BulletproofsBatchNatives => CedraFeatureFlag::BULLETPROOFS_BATCH_NATIVES,
            FeatureFlag::DerivableAccountAbstraction => {
                CedraFeatureFlag::DERIVABLE_ACCOUNT_ABSTRACTION
            },
            FeatureFlag::EnableFunctionValues => CedraFeatureFlag::ENABLE_FUNCTION_VALUES,
            FeatureFlag::NewAccountsDefaultToFaStore => {
                CedraFeatureFlag::NEW_ACCOUNTS_DEFAULT_TO_FA_STORE
            },
            FeatureFlag::DefaultAccountResource => CedraFeatureFlag::DEFAULT_ACCOUNT_RESOURCE,
            FeatureFlag::JwkConsensusPerKeyMode => CedraFeatureFlag::JWK_CONSENSUS_PER_KEY_MODE,
            FeatureFlag::TransactionPayloadV2 => CedraFeatureFlag::TRANSACTION_PAYLOAD_V2,
            FeatureFlag::OrderlessTransactions => CedraFeatureFlag::ORDERLESS_TRANSACTIONS,
            FeatureFlag::FeeV2 => CedraFeatureFlag::FEE_V2,
        }
    }
}

// We don't need this implementation. Just to make sure we have an exhaustive 1-1 mapping between the two structs.
impl From<CedraFeatureFlag> for FeatureFlag {
    fn from(f: CedraFeatureFlag) -> Self {
        match f {
            CedraFeatureFlag::CODE_DEPENDENCY_CHECK => FeatureFlag::CodeDependencyCheck,
            CedraFeatureFlag::_DEPRECATED_COLLECT_AND_DISTRIBUTE_GAS_FEES => {
                FeatureFlag::CollectAndDistributeGasFees
            },
            CedraFeatureFlag::TREAT_FRIEND_AS_PRIVATE => FeatureFlag::TreatFriendAsPrivate,
            CedraFeatureFlag::SHA_512_AND_RIPEMD_160_NATIVES => {
                FeatureFlag::Sha512AndRipeMd160Natives
            },
            CedraFeatureFlag::CEDRA_STD_CHAIN_ID_NATIVES => FeatureFlag::CedraStdChainIdNatives,
            CedraFeatureFlag::VM_BINARY_FORMAT_V6 => FeatureFlag::VMBinaryFormatV6,
            CedraFeatureFlag::VM_BINARY_FORMAT_V7 => FeatureFlag::VMBinaryFormatV7,
            CedraFeatureFlag::VM_BINARY_FORMAT_V8 => FeatureFlag::VMBinaryFormatV8,
            CedraFeatureFlag::MULTI_ED25519_PK_VALIDATE_V2_NATIVES => {
                FeatureFlag::MultiEd25519PkValidateV2Natives
            },
            CedraFeatureFlag::BLAKE2B_256_NATIVE => FeatureFlag::Blake2b256Native,
            CedraFeatureFlag::RESOURCE_GROUPS => FeatureFlag::ResourceGroups,
            CedraFeatureFlag::MULTISIG_ACCOUNTS => FeatureFlag::MultisigAccounts,
            CedraFeatureFlag::DELEGATION_POOLS => FeatureFlag::DelegationPools,
            CedraFeatureFlag::CRYPTOGRAPHY_ALGEBRA_NATIVES => {
                FeatureFlag::CryptographyAlgebraNatives
            },
            CedraFeatureFlag::BLS12_381_STRUCTURES => FeatureFlag::Bls12381Structures,
            CedraFeatureFlag::ED25519_PUBKEY_VALIDATE_RETURN_FALSE_WRONG_LENGTH => {
                FeatureFlag::Ed25519PubkeyValidateReturnFalseWrongLength
            },
            CedraFeatureFlag::STRUCT_CONSTRUCTORS => FeatureFlag::StructConstructors,
            CedraFeatureFlag::PERIODICAL_REWARD_RATE_DECREASE => {
                FeatureFlag::PeriodicalRewardRateReduction
            },
            CedraFeatureFlag::PARTIAL_GOVERNANCE_VOTING => FeatureFlag::PartialGovernanceVoting,
            CedraFeatureFlag::SIGNATURE_CHECKER_V2 => FeatureFlag::SignatureCheckerV2,
            CedraFeatureFlag::STORAGE_SLOT_METADATA => FeatureFlag::StorageSlotMetadata,
            CedraFeatureFlag::CHARGE_INVARIANT_VIOLATION => FeatureFlag::ChargeInvariantViolation,
            CedraFeatureFlag::DELEGATION_POOL_PARTIAL_GOVERNANCE_VOTING => {
                FeatureFlag::DelegationPoolPartialGovernanceVoting
            },
            CedraFeatureFlag::GAS_PAYER_ENABLED => FeatureFlag::GasPayerEnabled,
            CedraFeatureFlag::CEDRA_UNIQUE_IDENTIFIERS => FeatureFlag::CedraUniqueIdentifiers,
            CedraFeatureFlag::BULLETPROOFS_NATIVES => FeatureFlag::BulletproofsNatives,
            CedraFeatureFlag::SIGNER_NATIVE_FORMAT_FIX => FeatureFlag::SignerNativeFormatFix,
            CedraFeatureFlag::MODULE_EVENT => FeatureFlag::ModuleEvent,
            CedraFeatureFlag::EMIT_FEE_STATEMENT => FeatureFlag::EmitFeeStatement,
            CedraFeatureFlag::STORAGE_DELETION_REFUND => FeatureFlag::StorageDeletionRefund,
            CedraFeatureFlag::AGGREGATOR_V2_API => FeatureFlag::AggregatorV2Api,
            CedraFeatureFlag::SIGNATURE_CHECKER_V2_SCRIPT_FIX => {
                FeatureFlag::SignatureCheckerV2ScriptFix
            },
            CedraFeatureFlag::SAFER_RESOURCE_GROUPS => FeatureFlag::SaferResourceGroups,
            CedraFeatureFlag::SAFER_METADATA => FeatureFlag::SaferMetadata,
            CedraFeatureFlag::SINGLE_SENDER_AUTHENTICATOR => FeatureFlag::SingleSenderAuthenticator,
            CedraFeatureFlag::SPONSORED_AUTOMATIC_ACCOUNT_V1_CREATION => {
                FeatureFlag::SponsoredAutomaticAccountCreation
            },
            CedraFeatureFlag::FEE_PAYER_ACCOUNT_OPTIONAL => FeatureFlag::FeePayerAccountOptional,
            CedraFeatureFlag::AGGREGATOR_V2_DELAYED_FIELDS => {
                FeatureFlag::AggregatorV2DelayedFields
            },
            CedraFeatureFlag::CONCURRENT_TOKEN_V2 => FeatureFlag::ConcurrentTokenV2,
            CedraFeatureFlag::LIMIT_MAX_IDENTIFIER_LENGTH => FeatureFlag::LimitMaxIdentifierLength,
            CedraFeatureFlag::OPERATOR_BENEFICIARY_CHANGE => FeatureFlag::OperatorBeneficiaryChange,
            CedraFeatureFlag::RESOURCE_GROUPS_SPLIT_IN_VM_CHANGE_SET => {
                FeatureFlag::ResourceGroupsSplitInVmChangeSet
            },
            CedraFeatureFlag::COMMISSION_CHANGE_DELEGATION_POOL => {
                FeatureFlag::CommissionChangeDelegationPool
            },
            CedraFeatureFlag::BN254_STRUCTURES => FeatureFlag::Bn254Structures,
            CedraFeatureFlag::WEBAUTHN_SIGNATURE => FeatureFlag::WebAuthnSignature,
            CedraFeatureFlag::_DEPRECATED_RECONFIGURE_WITH_DKG => FeatureFlag::ReconfigureWithDkg,
            CedraFeatureFlag::KEYLESS_ACCOUNTS => FeatureFlag::KeylessAccounts,
            CedraFeatureFlag::KEYLESS_BUT_ZKLESS_ACCOUNTS => FeatureFlag::KeylessButZklessAccounts,
            CedraFeatureFlag::_DEPRECATED_REMOVE_DETAILED_ERROR_FROM_HASH => {
                FeatureFlag::RemoveDetailedError
            },
            CedraFeatureFlag::JWK_CONSENSUS => FeatureFlag::JwkConsensus,
            CedraFeatureFlag::CONCURRENT_FUNGIBLE_ASSETS => FeatureFlag::ConcurrentFungibleAssets,
            CedraFeatureFlag::REFUNDABLE_BYTES => FeatureFlag::RefundableBytes,
            CedraFeatureFlag::OBJECT_CODE_DEPLOYMENT => FeatureFlag::ObjectCodeDeployment,
            CedraFeatureFlag::MAX_OBJECT_NESTING_CHECK => FeatureFlag::MaxObjectNestingCheck,
            CedraFeatureFlag::KEYLESS_ACCOUNTS_WITH_PASSKEYS => {
                FeatureFlag::KeylessAccountsWithPasskeys
            },
            CedraFeatureFlag::MULTISIG_V2_ENHANCEMENT => FeatureFlag::MultisigV2Enhancement,
            CedraFeatureFlag::DELEGATION_POOL_ALLOWLISTING => {
                FeatureFlag::DelegationPoolAllowlisting
            },
            CedraFeatureFlag::MODULE_EVENT_MIGRATION => FeatureFlag::ModuleEventMigration,
            CedraFeatureFlag::_REJECT_UNSTABLE_BYTECODE => FeatureFlag::RejectUnstableBytecode,
            CedraFeatureFlag::TRANSACTION_CONTEXT_EXTENSION => {
                FeatureFlag::TransactionContextExtension
            },
            CedraFeatureFlag::COIN_TO_FUNGIBLE_ASSET_MIGRATION => {
                FeatureFlag::CoinToFungibleAssetMigration
            },
            CedraFeatureFlag::PRIMARY_CEDRA_FUNGIBLE_STORE_AT_USER_ADDRESS => {
                FeatureFlag::PrimaryCedraFungibleStoreAtUserAddress
            },
            CedraFeatureFlag::OBJECT_NATIVE_DERIVED_ADDRESS => {
                FeatureFlag::ObjectNativeDerivedAddress
            },
            CedraFeatureFlag::DISPATCHABLE_FUNGIBLE_ASSET => FeatureFlag::DispatchableFungibleAsset,
            CedraFeatureFlag::NEW_ACCOUNTS_DEFAULT_TO_FA_CEDRA_STORE => {
                FeatureFlag::NewAccountsDefaultToFaAptStore
            },
            CedraFeatureFlag::OPERATIONS_DEFAULT_TO_FA_CEDRA_STORE => {
                FeatureFlag::OperationsDefaultToFaAptStore
            },
            CedraFeatureFlag::AGGREGATOR_V2_IS_AT_LEAST_API => {
                FeatureFlag::AggregatorV2IsAtLeastApi
            },
            CedraFeatureFlag::CONCURRENT_FUNGIBLE_BALANCE => FeatureFlag::ConcurrentFungibleBalance,
            CedraFeatureFlag::DEFAULT_TO_CONCURRENT_FUNGIBLE_BALANCE => {
                FeatureFlag::DefaultToConcurrentFungibleBalance
            },
            CedraFeatureFlag::_LIMIT_VM_TYPE_SIZE => FeatureFlag::LimitVMTypeSize,
            CedraFeatureFlag::ABORT_IF_MULTISIG_PAYLOAD_MISMATCH => {
                FeatureFlag::AbortIfMultisigPayloadMismatch
            },
            CedraFeatureFlag::_DISALLOW_USER_NATIVES => FeatureFlag::DisallowUserNative,
            CedraFeatureFlag::ALLOW_SERIALIZED_SCRIPT_ARGS => {
                FeatureFlag::AllowSerializedScriptArgs
            },
            CedraFeatureFlag::_USE_COMPATIBILITY_CHECKER_V2 => {
                FeatureFlag::UseCompatibilityCheckerV2
            },
            CedraFeatureFlag::ENABLE_ENUM_TYPES => FeatureFlag::EnableEnumTypes,
            CedraFeatureFlag::ENABLE_RESOURCE_ACCESS_CONTROL => {
                FeatureFlag::EnableResourceAccessControl
            },
            CedraFeatureFlag::_REJECT_UNSTABLE_BYTECODE_FOR_SCRIPT => {
                FeatureFlag::RejectUnstableBytecodeForScript
            },
            CedraFeatureFlag::FEDERATED_KEYLESS => FeatureFlag::FederatedKeyless,
            CedraFeatureFlag::TRANSACTION_SIMULATION_ENHANCEMENT => {
                FeatureFlag::TransactionSimulationEnhancement
            },
            CedraFeatureFlag::COLLECTION_OWNER => FeatureFlag::CollectionOwner,
            CedraFeatureFlag::NATIVE_MEMORY_OPERATIONS => FeatureFlag::NativeMemoryOperations,
            CedraFeatureFlag::_ENABLE_LOADER_V2 => FeatureFlag::EnableLoaderV2,
            CedraFeatureFlag::_DISALLOW_INIT_MODULE_TO_PUBLISH_MODULES => {
                FeatureFlag::DisallowInitModuleToPublishModules
            },
            CedraFeatureFlag::ENABLE_CALL_TREE_AND_INSTRUCTION_VM_CACHE => {
                FeatureFlag::EnableCallTreeAndInstructionVMCache
            },
            CedraFeatureFlag::PERMISSIONED_SIGNER => FeatureFlag::PermissionedSigner,
            CedraFeatureFlag::ACCOUNT_ABSTRACTION => FeatureFlag::AccountAbstraction,
            CedraFeatureFlag::BULLETPROOFS_BATCH_NATIVES => FeatureFlag::BulletproofsBatchNatives,
            CedraFeatureFlag::DERIVABLE_ACCOUNT_ABSTRACTION => {
                FeatureFlag::DerivableAccountAbstraction
            },
            CedraFeatureFlag::ENABLE_FUNCTION_VALUES => FeatureFlag::EnableFunctionValues,
            CedraFeatureFlag::NEW_ACCOUNTS_DEFAULT_TO_FA_STORE => {
                FeatureFlag::NewAccountsDefaultToFaStore
            },
            CedraFeatureFlag::DEFAULT_ACCOUNT_RESOURCE => FeatureFlag::DefaultAccountResource,
            CedraFeatureFlag::JWK_CONSENSUS_PER_KEY_MODE => FeatureFlag::JwkConsensusPerKeyMode,
            CedraFeatureFlag::TRANSACTION_PAYLOAD_V2 => FeatureFlag::TransactionPayloadV2,
            CedraFeatureFlag::ORDERLESS_TRANSACTIONS => FeatureFlag::OrderlessTransactions,
            CedraFeatureFlag::FEE_V2 => FeatureFlag::FeeV2,
        }
    }
}

impl Features {
    // Compare if the current feature set is different from features that has been enabled on chain.
    pub(crate) fn has_modified(&self, on_chain_features: &CedraFeatures) -> bool {
        self.enabled
            .iter()
            .any(|f| !on_chain_features.is_enabled(CedraFeatureFlag::from(f.clone())))
            || self
                .disabled
                .iter()
                .any(|f| on_chain_features.is_enabled(CedraFeatureFlag::from(f.clone())))
    }
}

impl From<&CedraFeatures> for Features {
    fn from(features: &CedraFeatures) -> Features {
        let mut enabled = vec![];
        let mut disabled = vec![];
        for feature in FeatureFlag::iter() {
            if features.is_enabled(CedraFeatureFlag::from(feature.clone())) {
                enabled.push(feature);
            } else {
                disabled.push(feature);
            }
        }
        Features { enabled, disabled }
    }
}
