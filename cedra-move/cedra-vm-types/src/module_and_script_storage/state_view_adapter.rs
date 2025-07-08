// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::duplicated_attributes)]

use crate::{
    module_and_script_storage::module_storage::CedraModuleStorage,
    resolver::BlockSynchronizationKillSwitch,
};
use ambassador::Delegate;
use cedra_types::{
    error::PanicError,
    state_store::{state_key::StateKey, state_value::StateValueMetadata, StateView, TStateView},
    vm::modules::CedraModuleExtension,
};
use bytes::Bytes;
use move_binary_format::{
    errors::{PartialVMResult, VMResult},
    file_format::CompiledScript,
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    metadata::Metadata,
};
use move_vm_runtime::{
    ambassador_impl_CodeStorage, ambassador_impl_ModuleStorage,
    ambassador_impl_WithRuntimeEnvironment, AsUnsyncCodeStorage, CodeStorage, Function,
    LoadedFunction, Module, ModuleStorage, RuntimeEnvironment, Script, UnsyncCodeStorage,
    UnsyncModuleStorage, WithRuntimeEnvironment,
};
use move_vm_types::{
    code::{ModuleBytesStorage, ModuleCode},
    loaded_data::{
        runtime_types::{StructType, Type},
        struct_name_indexing::StructNameIndex,
    },
    module_storage_error,
};
use std::{ops::Deref, sync::Arc};

struct StateViewAdapter<'ctx, S, E> {
    environment: &'ctx E,
    state_view: &'ctx S,
}

impl<S, E> WithRuntimeEnvironment for StateViewAdapter<'_, S, E>
where
    S: StateView,
    E: WithRuntimeEnvironment,
{
    fn runtime_environment(&self) -> &RuntimeEnvironment {
        self.environment.runtime_environment()
    }
}

impl<S, E> ModuleBytesStorage for StateViewAdapter<'_, S, E>
where
    S: StateView,
    E: WithRuntimeEnvironment,
{
    fn fetch_module_bytes(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> VMResult<Option<Bytes>> {
        let state_key = StateKey::module(address, module_name);
        self.state_view
            .get_state_value_bytes(&state_key)
            .map_err(|e| module_storage_error!(address, module_name, e))
    }
}

impl<S, E> Deref for StateViewAdapter<'_, S, E>
where
    S: StateView,
    E: WithRuntimeEnvironment,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.state_view
    }
}

/// A (not thread-safe) implementation of code storage on top of a state view. It is never built
/// directly by clients - only via [AsCedraCodeStorage] trait. Can be used to resolve both modules
/// and cached scripts.
#[derive(Delegate)]
#[delegate(
    WithRuntimeEnvironment,
    where = "S: StateView, E: WithRuntimeEnvironment"
)]
#[delegate(ModuleStorage, where = "S: StateView, E: WithRuntimeEnvironment")]
#[delegate(CodeStorage, where = "S: StateView, E: WithRuntimeEnvironment")]
pub struct CedraCodeStorageAdapter<'ctx, S, E> {
    storage: UnsyncCodeStorage<UnsyncModuleStorage<'ctx, StateViewAdapter<'ctx, S, E>>>,
}

impl<S, E> CedraCodeStorageAdapter<'_, S, E>
where
    S: StateView,
    E: WithRuntimeEnvironment,
{
    /// Drains cached verified modules from the code storage, transforming them into format used by
    /// global caches.
    pub fn into_verified_module_code_iter(
        self,
    ) -> Result<
        impl Iterator<
            Item = (
                ModuleId,
                Arc<ModuleCode<CompiledModule, Module, CedraModuleExtension>>,
            ),
        >,
        PanicError,
    > {
        let (state_view, verified_modules_iter) = self
            .storage
            .into_module_storage()
            .unpack_into_verified_modules_iter();

        Ok(verified_modules_iter
            .map(|(key, verified_code)| {
                // We have cached the module previously, so we must be able to find it in storage.
                let extension = state_view
                    .get_state_value(&StateKey::module_id(&key))
                    .map_err(|err| {
                        let msg = format!(
                            "Failed to retrieve module {}::{} from storage {:?}",
                            key.address(),
                            key.name(),
                            err
                        );
                        PanicError::CodeInvariantError(msg)
                    })?
                    .map_or_else(
                        || {
                            let msg = format!(
                                "Module {}::{} should exist, but it does not anymore",
                                key.address(),
                                key.name()
                            );
                            Err(PanicError::CodeInvariantError(msg))
                        },
                        |state_value| Ok(CedraModuleExtension::new(state_value)),
                    )?;

                let module = ModuleCode::from_arced_verified(verified_code, Arc::new(extension));
                Ok((key, Arc::new(module)))
            })
            .collect::<Result<Vec<_>, PanicError>>()?
            .into_iter())
    }
}

impl<S: StateView, E: WithRuntimeEnvironment> CedraModuleStorage
    for CedraCodeStorageAdapter<'_, S, E>
{
    fn fetch_state_value_metadata(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> PartialVMResult<Option<StateValueMetadata>> {
        let state_key = StateKey::module(address, module_name);
        Ok(self
            .storage
            .module_storage()
            .byte_storage()
            .state_view
            .get_state_value(&state_key)
            .map_err(|err| module_storage_error!(address, module_name, err).to_partial())?
            .map(|state_value| state_value.into_metadata()))
    }
}

impl<S: StateView, E: WithRuntimeEnvironment> BlockSynchronizationKillSwitch
    for CedraCodeStorageAdapter<'_, S, E>
{
    fn interrupt_requested(&self) -> bool {
        false
    }
}

/// Allows to treat the state view as a code storage with scripts and modules. The main use case is
/// when a transaction or a Move function has to be executed outside the long-living environment or
/// block executor, e.g., for single transaction simulation, in Cedra debugger, etc.
pub trait AsCedraCodeStorage<'ctx, S, E> {
    fn as_cedra_code_storage(
        &'ctx self,
        environment: &'ctx E,
    ) -> CedraCodeStorageAdapter<'ctx, S, E>;
}

impl<'ctx, S, E> AsCedraCodeStorage<'ctx, S, E> for S
where
    S: StateView,
    E: WithRuntimeEnvironment,
{
    fn as_cedra_code_storage(
        &'ctx self,
        environment: &'ctx E,
    ) -> CedraCodeStorageAdapter<'ctx, S, E> {
        let adapter = StateViewAdapter {
            environment,
            state_view: self,
        };
        let storage = adapter.into_unsync_code_storage();
        CedraCodeStorageAdapter { storage }
    }
}
