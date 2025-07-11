// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use cedra_logger::Schema;
use serde::Serialize;

#[derive(Schema)]
pub struct LogSchema<'a> {
    name: LogEntry,
    error: Option<&'a Error>,
    event: Option<LogEvent>,
    message: Option<&'a str>,
    stream_id: Option<u64>,
}

impl<'a> LogSchema<'a> {
    pub fn new(name: LogEntry) -> Self {
        Self {
            name,
            error: None,
            event: None,
            message: None,
            stream_id: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    CedraDataClient,
    CheckStreamProgress,
    CreatedSubscriptionStream,
    EndOfStreamNotification,
    HandleStreamRequest,
    HandleTerminateRequest,
    InitializeStream,
    ReceivedDataResponse,
    RefreshGlobalData,
    RequestError,
    RespondToStreamRequest,
    RetryDataRequest,
    SendDataRequests,
    StreamNotification,
    TerminateStream,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    Error,
    Pending,
    Success,
}
