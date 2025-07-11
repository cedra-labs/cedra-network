// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for any single-entry data.
//!
//! There will be only one row in this column family for each type of data.
//! The key will be a serialized enum type designating the data type and should not have any meaning
//! and be used.
//!
//! ```text
//! |<-------key------->|<-----value----->|
//! | single entry key  | raw value bytes |
//! ```

use super::ensure_slice_len_eq;
use crate::define_schema;
use anyhow::{format_err, Result};
use cedra_schemadb::{
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};
use byteorder::ReadBytesExt;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::mem::size_of;

pub const SINGLE_ENTRY_CF_NAME: ColumnFamilyName = "single_entry";

define_schema!(
    SingleEntrySchema,
    SingleEntryKey,
    Vec<u8>,
    SINGLE_ENTRY_CF_NAME
);

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum SingleEntryKey {
    // Used to store the last vote
    LastVote = 0,
    // Two chain timeout cert
    Highest2ChainTimeoutCert = 1,
}

impl KeyCodec<SingleEntrySchema> for SingleEntryKey {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(vec![self
            .to_u8()
            .ok_or_else(|| format_err!("ToPrimitive failed."))?])
    }

    fn decode_key(mut data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<u8>())?;
        let key = data.read_u8()?;
        SingleEntryKey::from_u8(key).ok_or_else(|| format_err!("FromPrimitive failed."))
    }
}

impl ValueCodec<SingleEntrySchema> for Vec<u8> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.clone())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod test;
