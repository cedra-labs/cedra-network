// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::schema::{
    db_metadata::DbMetadataKey, stale_node_index::StaleNodeIndexSchema,
    stale_node_index_cross_epoch::StaleNodeIndexCrossEpochSchema,
};
use cedra_jellyfish_merkle::StaleNodeIndex;
use cedra_schemadb::schema::{KeyCodec, Schema};

pub trait StaleNodeIndexSchemaTrait: Schema<Key = StaleNodeIndex>
where
    StaleNodeIndex: KeyCodec<Self>,
{
    fn progress_metadata_key(shard_id: Option<u8>) -> DbMetadataKey;
    fn name() -> &'static str;
}

impl StaleNodeIndexSchemaTrait for StaleNodeIndexSchema {
    fn progress_metadata_key(shard_id: Option<u8>) -> DbMetadataKey {
        if let Some(shard_id) = shard_id {
            DbMetadataKey::StateMerkleShardPrunerProgress(shard_id as usize)
        } else {
            DbMetadataKey::StateMerklePrunerProgress
        }
    }

    fn name() -> &'static str {
        "state_merkle_pruner"
    }
}

impl StaleNodeIndexSchemaTrait for StaleNodeIndexCrossEpochSchema {
    fn progress_metadata_key(shard_id: Option<u8>) -> DbMetadataKey {
        if let Some(shard_id) = shard_id {
            DbMetadataKey::EpochEndingStateMerkleShardPrunerProgress(shard_id as usize)
        } else {
            DbMetadataKey::EpochEndingStateMerklePrunerProgress
        }
    }

    fn name() -> &'static str {
        "epoch_snapshot_pruner"
    }
}
