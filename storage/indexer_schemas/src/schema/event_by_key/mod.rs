// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for an event index via which a ContractEvent (
//! represented by a <txn_version, event_idx> tuple so that it can be fetched from `EventSchema`)
//! can be found by <access_path, sequence_num> tuple.
//!
//! ```text
//! |<---------key------->|<----value---->|
//! | event_key | seq_num | txn_ver | idx |
//! ```

use crate::{schema::EVENT_BY_KEY_CF_NAME, utils::ensure_slice_len_eq};
use anyhow::Result;
use cedra_schemadb::{
    define_pub_schema,
    schema::{KeyCodec, ValueCodec},
};
use cedra_types::{event::EventKey, transaction::Version};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::mem::size_of;

define_pub_schema!(EventByKeySchema, Key, Value, EVENT_BY_KEY_CF_NAME);

type SeqNum = u64;
type Key = (EventKey, SeqNum);

type Index = u64;
type Value = (Version, Index);

impl KeyCodec<EventByKeySchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (ref event_key, seq_num) = *self;

        let mut encoded = event_key.to_bytes();
        encoded.write_u64::<BigEndian>(seq_num)?;

        Ok(encoded)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        const EVENT_KEY_LEN: usize = size_of::<EventKey>();
        let event_key = bcs::from_bytes(&data[..EVENT_KEY_LEN])?;
        let seq_num = (&data[EVENT_KEY_LEN..]).read_u64::<BigEndian>()?;

        Ok((event_key, seq_num))
    }
}

impl ValueCodec<EventByKeySchema> for Value {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let (version, index) = *self;

        let mut encoded = Vec::with_capacity(size_of::<Version>() + size_of::<Index>());
        encoded.write_u64::<BigEndian>(version)?;
        encoded.write_u64::<BigEndian>(index)?;

        Ok(encoded)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        const VERSION_SIZE: usize = size_of::<Version>();
        let version = (&data[..VERSION_SIZE]).read_u64::<BigEndian>()?;
        let index = (&data[VERSION_SIZE..]).read_u64::<BigEndian>()?;

        Ok((version, index))
    }
}

#[cfg(test)]
mod test;
