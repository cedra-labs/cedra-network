// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_framework::ReleaseBundle;
use once_cell::sync::Lazy;

pub mod cedra_framework_sdk_builder;
pub mod cedra_stdlib;
pub mod cedra_token_objects_sdk_builder;
pub mod cedra_token_sdk_builder;

#[cfg(unix)]
const HEAD_RELEASE_BUNDLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/head.mrb"));
#[cfg(windows)]
const HEAD_RELEASE_BUNDLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\head.mrb"));

static HEAD_RELEASE_BUNDLE: Lazy<ReleaseBundle> = Lazy::new(|| {
    bcs::from_bytes::<ReleaseBundle>(HEAD_RELEASE_BUNDLE_BYTES).expect("bcs succeeds")
});

/// Returns the release bundle for the current code.
pub fn head_release_bundle() -> &'static ReleaseBundle {
    &HEAD_RELEASE_BUNDLE
}
