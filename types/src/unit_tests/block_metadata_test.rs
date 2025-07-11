// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::block_metadata::BlockMetadata;
use bcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_block_metadata_canonical_serialization(data in any::<BlockMetadata>()) {
        assert_canonical_encode_decode(data);
    }
}
