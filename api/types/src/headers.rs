// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

/// Chain ID of the current chain
pub const X_CEDRA_CHAIN_ID: &str = "X-Cedra-Chain-Id";
/// Current epoch of the chain
pub const X_CEDRA_EPOCH: &str = "X-Cedra-Epoch";
/// Current ledger version of the chain
pub const X_CEDRA_LEDGER_VERSION: &str = "X-Cedra-Ledger-Version";
/// Oldest non-pruned ledger version of the chain
pub const X_CEDRA_LEDGER_OLDEST_VERSION: &str = "X-Cedra-Ledger-Oldest-Version";
/// Current block height of the chain
pub const X_CEDRA_BLOCK_HEIGHT: &str = "X-Cedra-Block-Height";
/// Oldest non-pruned block height of the chain
pub const X_CEDRA_OLDEST_BLOCK_HEIGHT: &str = "X-Cedra-Oldest-Block-Height";
/// Current timestamp of the chain
pub const X_CEDRA_LEDGER_TIMESTAMP: &str = "X-Cedra-Ledger-TimestampUsec";
/// Cursor used for pagination.
pub const X_CEDRA_CURSOR: &str = "X-Cedra-Cursor";
/// The cost of the call in terms of gas. Only applicable to calls that result in
/// function execution in the VM, e.g. view functions, txn simulation.
pub const X_CEDRA_GAS_USED: &str = "X-Cedra-Gas-Used";
/// Provided by the client to identify what client it is.
pub const X_CEDRA_CLIENT: &str = "x-cedra-client";
