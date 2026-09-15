// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_vm_ext::{CedraMoveResolver, SessionId},
    CedraVM,
};
use cedra_types::validator_txn::ValidatorTransaction;
use cedra_vm_logging::log_schema::AdapterLogSchema;
use cedra_vm_types::{
    module_and_script_storage::module_storage::CedraModuleStorage, output::VMOutput,
};
use move_core_types::vm_status::VMStatus;

impl CedraVM {
    pub(crate) fn process_validator_transaction(
        &self,
        resolver: &impl CedraMoveResolver,
        module_storage: &impl CedraModuleStorage,
        txn: ValidatorTransaction,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, VMOutput), VMStatus> {
        let session_id = SessionId::validator_txn(&txn);
        match txn {
            ValidatorTransaction::DKGResult(dkg_node) => {
                self.process_dkg_result(resolver, module_storage, log_context, session_id, dkg_node)
            },
            ValidatorTransaction::ObservedJWKUpdate(jwk_update) => self.process_jwk_update(
                resolver,
                module_storage,
                log_context,
                session_id,
                jwk_update,
            ),
            ValidatorTransaction::AddPrice(price_info) => self.process_price_add(
                resolver,
                module_storage,
                log_context,
                session_id,
                price_info,
            ),
            ValidatorTransaction::RemovePrice(fa_address) => self.process_price_remove(
                resolver,
                module_storage,
                log_context,
                session_id,
                fa_address,
            ),
        }
    }
}

mod dkg;
mod jwk;
mod oracle;
