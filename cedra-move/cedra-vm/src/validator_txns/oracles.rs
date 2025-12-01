// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cedra_vm::get_system_transaction_output,
    errors::expect_only_successful_execution,
    move_vm_ext::{CedraMoveResolver, SessionId},
    system_module_names::{PRICE_STORAGE_MODULE, REMOVE_PRICE, SET_PRICE},
    validator_txns::oracles::ExecutionFailure::{Expected, Unexpected},
    CedraVM,
};

use cedra_logger::debug;
use cedra_types::{move_utils::as_move_value::AsMoveValue, oracles::PriceInfo, transaction::TransactionStatus};
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

#[derive(Debug)]
#[allow(dead_code)]
enum ExpectedFailure {
    // Move equivalent: `errors::invalid_argument(*)`
    IncorrectVersion = 0x010103,
    MultiSigVerificationFailed = 0x010104,
    NotEnoughVotingPower = 0x010105,
}

#[allow(dead_code)]
enum ExecutionFailure {
    Expected(ExpectedFailure),
    Unexpected(VMStatus),
}

impl CedraVM {
    pub(crate) fn process_price_update(
        &self,
        resolver: &impl CedraMoveResolver,
        module_storage: &impl CedraModuleStorage,
        log_context: &AdapterLogSchema,
        session_id: SessionId,
        update: PriceInfo,
    ) -> Result<(VMStatus, VMOutput), VMStatus> {
        debug!("Processing price update transaction");
        match self.process_price_storage_update_inner(
            resolver,
            module_storage,
            log_context,
            session_id,
            update,
        ) {
            Ok((vm_status, vm_output)) => {
                debug!("Processing price_storage transaction ok.");
                Ok((vm_status, vm_output))
            },
            Err(Expected(failure)) => {
                debug!(
                    "Processing price_storage transaction expected failure: {:?}",
                    failure
                );
                Ok((
                    VMStatus::MoveAbort(AbortLocation::Script, failure as u64),
                    VMOutput::empty_with_status(TransactionStatus::Discard(StatusCode::ABORTED)),
                ))
            },
            Err(Unexpected(vm_status)) => {
                debug!(
                    "Processing price_storage transaction unexpected failure: {:?}",
                    vm_status
                );
                Err(vm_status)
            },
        }
    }

    pub(crate) fn _process_price_remove(
        &self,
        resolver: &impl CedraMoveResolver,
        module_storage: &impl CedraModuleStorage,
        log_context: &AdapterLogSchema,
        session_id: SessionId,
        fa_address: AccountAddress,
    ) -> Result<(VMStatus, VMOutput), VMStatus> {
        debug!("Processing price remove transaction");
        match self.process_price_storage_remove_inner(
            resolver,
            module_storage,
            log_context,
            session_id,
            fa_address,
        ) {
            Ok((vm_status, vm_output)) => {
                debug!("Processing price_storage remove ok.");
                Ok((vm_status, vm_output))
            },
            Err(Expected(failure)) => {
                debug!(
                    "Processing price_storage remove expected failure: {:?}",
                    failure
                );
                Ok((
                    VMStatus::MoveAbort(AbortLocation::Script, failure as u64),
                    VMOutput::empty_with_status(TransactionStatus::Discard(StatusCode::ABORTED)),
                ))
            },
            Err(Unexpected(vm_status)) => {
                debug!(
                    "Processing price_storage remove unexpected failure: {:?}",
                    vm_status
                );
                Err(vm_status)
            },
        }
    }

    fn process_price_storage_update_inner(
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
            price_info.as_move_value()
                    ];

        let traversal_storage = TraversalStorage::new();
        session
            .execute_function_bypass_visibility(
                &PRICE_STORAGE_MODULE,
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

    #[allow(dead_code)]
    fn process_price_storage_remove_inner(
        &self,
        resolver: &impl CedraMoveResolver,
        module_storage: &impl CedraModuleStorage,
        log_context: &AdapterLogSchema,
        session_id: SessionId,
        fa_address: AccountAddress,
    ) -> Result<(VMStatus, VMOutput), ExecutionFailure> {
        let mut gas_meter = UnmeteredGasMeter;
        let mut session = self.new_session(resolver, session_id, None);
        let args = vec![
            MoveValue::Signer(AccountAddress::ONE),
            MoveValue::Address(fa_address),
        ];

        let traversal_storage = TraversalStorage::new();
        session
            .execute_function_bypass_visibility(
                &PRICE_STORAGE_MODULE,
                REMOVE_PRICE,
                vec![],
                serialize_values(&args),
                &mut gas_meter,
                &mut TraversalContext::new(&traversal_storage),
                module_storage,
            )
            .map_err(|e| expect_only_successful_execution(e, REMOVE_PRICE.as_str(), log_context))
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
