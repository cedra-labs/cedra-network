// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{Metadata, Schema};
use std::fmt;

/// An individual structured logging event from a log line.  Includes the
#[derive(Debug)]
pub struct Event<'a> {
    metadata: &'a Metadata,
    /// The format message given from the log macros
    message: Option<fmt::Arguments<'a>>,
    keys_and_values: KeysAndValues<'a>,
}

impl<'a> Event<'a> {
    pub(crate) fn new(
        metadata: &'a Metadata,
        message: Option<fmt::Arguments<'a>>,
        keys_and_values: &'a [&'a dyn Schema],
    ) -> Self {
        Self {
            metadata,
            message,
            keys_and_values: KeysAndValues(keys_and_values),
        }
    }

    pub fn dispatch(
        metadata: &'a Metadata,
        message: Option<fmt::Arguments<'a>>,
        keys_and_values: &'a [&'a dyn Schema],
    ) {
        let event = Event::new(metadata, message, keys_and_values);
        crate::logger::dispatch(&event)
    }

    pub fn metadata(&self) -> &'a Metadata {
        self.metadata
    }

    pub fn message(&self) -> Option<fmt::Arguments<'a>> {
        self.message
    }

    pub fn keys_and_values(&self) -> &'a [&'a dyn Schema] {
        self.keys_and_values.0
    }
}

/// Keys and values given from the log `a = b` macros
#[derive(Clone)]
struct KeysAndValues<'a>(&'a [&'a dyn Schema]);

impl<'a> fmt::Debug for KeysAndValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut visitor = f.debug_map();
        for key_value in self.0 {
            key_value.visit(&mut visitor);
        }
        visitor.finish()
    }
}
