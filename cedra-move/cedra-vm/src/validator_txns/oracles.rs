// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cedra_vm::get_system_transaction_output,
    errors::expect_only_successful_execution,
    move_vm_ext::{CedraMoveResolver, SessionId},
    system_module_names::{PRICES_STORAGE_MODULE, SET_PRICE},
    validator_txns::oracles::{
        ExecutionFailure::{Expected, Unexpected},
        ExpectedFailure::{
            IncorrectVersion, MissingResourceObservedJWKs, MissingResourceValidatorSet,
            MultiSigVerificationFailed, NotEnoughVotingPower,
        },
    },
    CedraVM,
};
use cedra_logger::debug;
use cedra_types::{
    move_utils::as_move_value::AsMoveValue,
    on_chain_config::{OnChainConfig, ValidatorSet},
    oracles::PriceInfo,
    transaction::TransactionStatus,
    validator_verifier::ValidatorVerifier,
};
use cedra_vm_logging::log_schema::AdapterLogSchema;
use cedra_vm_types::{
    module_and_script_storage::module_storage::CedraModuleStorage, output::VMOutput,
};
use move_core_types::{
    account_address::AccountAddress,
    value::{serialize_values, MoveValue},
    vm_status::{AbortLocation, StatusCode, VMStatus},
};
use move_vm_runtime::module_traversal::{TraversalContext, TraversalStorage};
use move_vm_types::gas::UnmeteredGasMeter;
use std::collections::HashMap;

#[derive(Debug)]
enum ExpectedFailure {
    // Move equivalent: `errors::invalid_argument(*)`
    IncorrectVersion = 0x010103,
    MultiSigVerificationFailed = 0x010104,
    NotEnoughVotingPower = 0x010105,

    // Move equivalent: `errors::invalid_state(*)`
    MissingResourceValidatorSet = 0x30101,
    MissingResourceObservedJWKs = 0x30102,
}

enum ExecutionFailure {
    Expected(ExpectedFailure),
    Unexpected(VMStatus),
}

impl CedraVM {
    pub(crate) fn process_prices_storage_update(
        &self,
        resolver: &impl CedraMoveResolver,
        module_storage: &impl CedraModuleStorage,
        log_context: &AdapterLogSchema,
        session_id: SessionId,
        update: PriceInfo,
    ) -> Result<(VMStatus, VMOutput), VMStatus> {
        debug!("Processing price update transaction");
        match self.process_prices_storage_update_inner(
            resolver,
            module_storage,
            log_context,
            session_id,
            update,
        ) {
            Ok((vm_status, vm_output)) => {
                debug!("Processing prices_storage transaction ok.");
                Ok((vm_status, vm_output))
            },
            Err(Expected(failure)) => {
                // Pretend we are inside Move, and expected failures are like Move aborts.
                debug!(
                    "Processing prices_storage transaction expected failure: {:?}",
                    failure
                );
                Ok((
                    VMStatus::MoveAbort(AbortLocation::Script, failure as u64),
                    VMOutput::empty_with_status(TransactionStatus::Discard(StatusCode::ABORTED)),
                ))
            },
            Err(Unexpected(vm_status)) => {
                debug!(
                    "Processing prices_storage transaction unexpected failure: {:?}",
                    vm_status
                );
                Err(vm_status)
            },
        }
    }

    fn process_prices_storage_update_inner(
        &self,
        resolver: &impl CedraMoveResolver,
        module_storage: &impl CedraModuleStorage,
        log_context: &AdapterLogSchema,
        session_id: SessionId,
        price_info: PriceInfo,
    ) -> Result<(VMStatus, VMOutput), ExecutionFailure> {
        let mut gas_meter = UnmeteredGasMeter;
        let mut session = self.new_session(resolver, session_id, None);
        let args = vec![
            MoveValue::Signer(AccountAddress::ONE),
            MoveValue::Address(PriceInfo::to_move_address(&price_info)),
            MoveValue::U64(price_info.price),
            MoveValue::U8(price_info.decimals),
        ];

        let traversal_storage = TraversalStorage::new();
        session
            .execute_function_bypass_visibility(
                &PRICES_STORAGE_MODULE,
                SET_PRICE,
                vec![],
                serialize_values(&args),
                &mut gas_meter,
                &mut TraversalContext::new(&traversal_storage),
                module_storage,
            )
            .map_err(|e| expect_only_successful_execution(e, SET_PRICE.as_str(), log_context))
            .map_err(|r| Unexpected(r.unwrap_err()))?;

        let output = get_system_transaction_output(
            session,
            module_storage,
            &self
                .storage_gas_params(log_context)
                .map_err(Unexpected)?
                .change_set_configs,
        )
        .map_err(Unexpected)?;

        Ok((VMStatus::Executed, output))
    }
}
