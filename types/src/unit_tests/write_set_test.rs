// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::write_set::WriteSet;
use bcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn write_set_roundtrip_canonical_serialization(write_set in any::<WriteSet>()) {
        assert_canonical_encode_decode(write_set);
    }
}
