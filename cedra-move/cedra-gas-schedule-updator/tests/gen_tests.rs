// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_framework::{BuildOptions, BuiltPackage};
use cedra_gas_schedule_updator::{generate_update_proposal, GenArgs};

#[test]
fn can_generate_and_build_update_proposal() {
    let output_dir = tempfile::tempdir().unwrap();

    generate_update_proposal(&GenArgs {
        gas_feature_version: None,
        output: Some(output_dir.path().to_string_lossy().to_string()),
    })
    .unwrap();

    BuiltPackage::build(output_dir.path().to_path_buf(), BuildOptions::default()).unwrap();
}
