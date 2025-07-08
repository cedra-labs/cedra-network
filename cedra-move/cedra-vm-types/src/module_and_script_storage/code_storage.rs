// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::module_and_script_storage::module_storage::CedraModuleStorage;
use move_vm_runtime::CodeStorage;

/// Represents code storage used by the Cedra blockchain, capable of caching scripts and modules.
pub trait CedraCodeStorage: CedraModuleStorage + CodeStorage {}

impl<T: CedraModuleStorage + CodeStorage> CedraCodeStorage for T {}
