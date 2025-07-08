// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the version of Cedra Validator software.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CedraVersion {
    pub major: u64,
}

impl OnChainConfig for CedraVersion {
    const MODULE_IDENTIFIER: &'static str = "version";
    const TYPE_IDENTIFIER: &'static str = "Version";
}

// NOTE: version number for release 1.2 Cedra
// Items gated by this version number include:
//  - the EntryFunction payload type
pub const CEDRA_VERSION_2: CedraVersion = CedraVersion { major: 2 };

// NOTE: version number for release 1.3 of Cedra
// Items gated by this version number include:
//  - Multi-agent transactions
pub const CEDRA_VERSION_3: CedraVersion = CedraVersion { major: 3 };

// NOTE: version number for release 1.4 of Cedra
// Items gated by this version number include:
//  - Conflict-Resistant Sequence Numbers
pub const CEDRA_VERSION_4: CedraVersion = CedraVersion { major: 4 };

// Maximum current known version
pub const CEDRA_MAX_KNOWN_VERSION: CedraVersion = CEDRA_VERSION_4;
