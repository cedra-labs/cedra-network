// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OracleConfig {
    pub auth_key: Option<String>,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self { auth_key: None }
    }
}
