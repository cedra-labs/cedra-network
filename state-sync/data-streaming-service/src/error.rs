// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use futures::channel::{mpsc::SendError, oneshot::Canceled};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Eq, Serialize)]
pub enum Error {
    #[error("The requested data is unavailable and cannot be found in the network! Error: {0}")]
    DataIsUnavailable(String),
    #[error("Error returned by the cedra data client: {0}")]
    CedraDataClientError(String),
    #[error("The response from the cedra data client is invalid! Error: {0}")]
    CedraDataClientResponseIsInvalid(String),
    #[error("An integer overflow has occurred: {0}")]
    IntegerOverflow(String),
    #[error("No data to fetch: {0}")]
    NoDataToFetch(String),
    #[error("Unexpected error encountered: {0}")]
    UnexpectedErrorEncountered(String),
    #[error("Encountered an unsupported request: {0}")]
    UnsupportedRequestEncountered(String),
}

impl Error {
    /// Returns a summary label for the error
    pub fn get_label(&self) -> &'static str {
        match self {
            Self::DataIsUnavailable(_) => "data_is_unavailable",
            Self::CedraDataClientError(_) => "cedra_data_client_error",
            Self::CedraDataClientResponseIsInvalid(_) => "cedra_data_client_response_is_invalid",
            Self::IntegerOverflow(_) => "integer_overflow",
            Self::NoDataToFetch(_) => "no_data_to_fetch",
            Self::UnexpectedErrorEncountered(_) => "unexpected_error_encountered",
            Self::UnsupportedRequestEncountered(_) => "unsupported_request_encountered",
        }
    }
}

impl From<cedra_data_client::error::Error> for Error {
    fn from(error: cedra_data_client::error::Error) -> Self {
        Error::CedraDataClientError(error.to_string())
    }
}

impl From<SendError> for Error {
    fn from(error: SendError) -> Self {
        Error::UnexpectedErrorEncountered(error.to_string())
    }
}

impl From<Canceled> for Error {
    fn from(canceled: Canceled) -> Self {
        Error::UnexpectedErrorEncountered(canceled.to_string())
    }
}
