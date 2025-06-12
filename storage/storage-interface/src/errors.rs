// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This module defines error types used by `CedraDB`.
use cedra_types::state_store::errors::StateViewError;
use std::sync::mpsc::RecvError;
use thiserror::Error;

/// This enum defines errors commonly used among `CedraDB` APIs.
#[derive(Debug, Error)]
pub enum CedraDbError {
    /// A requested item is not found.
    #[error("{0} not found.")]
    NotFound(String),
    /// Requested too many items.
    #[error("Too many items requested: at least {0} requested, max is {1}")]
    TooManyRequested(u64, u64),
    #[error("Missing state root node at version {0}, probably pruned.")]
    MissingRootError(u64),
    /// Other non-classified error.
    #[error("CedraDB Other Error: {0}")]
    Other(String),
    #[error("CedraDB RocksDb Error: {0}")]
    RocksDbIncompleteResult(String),
    #[error("CedraDB RocksDB Error: {0}")]
    OtherRocksDbError(String),
    #[error("CedraDB bcs Error: {0}")]
    BcsError(String),
    #[error("CedraDB IO Error: {0}")]
    IoError(String),
    #[error("CedraDB Recv Error: {0}")]
    RecvError(String),
    #[error("CedraDB ParseInt Error: {0}")]
    ParseIntError(String),
}

impl From<anyhow::Error> for CedraDbError {
    fn from(error: anyhow::Error) -> Self {
        Self::Other(format!("{}", error))
    }
}

impl From<bcs::Error> for CedraDbError {
    fn from(error: bcs::Error) -> Self {
        Self::BcsError(format!("{}", error))
    }
}

impl From<RecvError> for CedraDbError {
    fn from(error: RecvError) -> Self {
        Self::RecvError(format!("{}", error))
    }
}

impl From<std::io::Error> for CedraDbError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(format!("{}", error))
    }
}

impl From<std::num::ParseIntError> for CedraDbError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::Other(format!("{}", error))
    }
}

impl From<CedraDbError> for StateViewError {
    fn from(error: CedraDbError) -> Self {
        match error {
            CedraDbError::NotFound(msg) => StateViewError::NotFound(msg),
            CedraDbError::Other(msg) => StateViewError::Other(msg),
            _ => StateViewError::Other(format!("{}", error)),
        }
    }
}

impl From<StateViewError> for CedraDbError {
    fn from(error: StateViewError) -> Self {
        match error {
            StateViewError::NotFound(msg) => CedraDbError::NotFound(msg),
            StateViewError::Other(msg) => CedraDbError::Other(msg),
            StateViewError::BcsError(err) => CedraDbError::BcsError(err.to_string()),
        }
    }
}
