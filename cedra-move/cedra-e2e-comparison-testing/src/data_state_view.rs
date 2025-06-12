// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_transaction_simulation::InMemoryStateStore;
use cedra_types::{
    state_store::{
        state_key::StateKey, state_storage_usage::StateStorageUsage, state_value::StateValue,
        StateViewResult, TStateView,
    },
    transaction::Version,
};
use cedra_validator_interface::{CedraValidatorInterface, DebuggerStateView};
use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

pub struct DataStateView {
    debugger_view: DebuggerStateView,
    code_data: Option<InMemoryStateStore>,
    data_read_state_keys: Option<Arc<Mutex<HashMap<StateKey, StateValue>>>>,
}

impl DataStateView {
    pub fn new(
        db: Arc<dyn CedraValidatorInterface + Send>,
        version: Version,
        code_data: InMemoryStateStore,
    ) -> Self {
        Self {
            debugger_view: DebuggerStateView::new(db, version),
            code_data: Some(code_data),
            data_read_state_keys: None,
        }
    }

    pub fn new_with_data_reads(
        db: Arc<dyn CedraValidatorInterface + Send>,
        version: Version,
    ) -> Self {
        Self {
            debugger_view: DebuggerStateView::new(db, version),
            code_data: None,
            data_read_state_keys: Some(Arc::new(Mutex::new(HashMap::new()))),
        }
    }

    pub fn get_state_keys(self) -> Arc<Mutex<HashMap<StateKey, StateValue>>> {
        self.data_read_state_keys.unwrap()
    }
}

impl TStateView for DataStateView {
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> StateViewResult<Option<StateValue>> {
        if let Some(code) = &self.code_data {
            if code.contains_state_value(state_key)? {
                return code.get_state_value(state_key).map_err(Into::into);
            }
        }
        let ret = self.debugger_view.get_state_value(state_key);
        if let Some(reads) = &self.data_read_state_keys {
            if !state_key.is_cedra_code()
                && !reads.lock().unwrap().contains_key(state_key)
                && ret.is_ok()
            {
                let val = ret?;
                if val.is_some() {
                    reads
                        .lock()
                        .unwrap()
                        .deref_mut()
                        .insert(state_key.clone(), val.clone().unwrap());
                };
                return Ok(val);
            }
        }
        ret
    }

    fn get_usage(&self) -> StateViewResult<StateStorageUsage> {
        unreachable!()
    }
}
