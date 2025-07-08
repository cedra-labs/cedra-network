// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_api::context::Context;
use cedra_config::config::NodeConfig;
use cedra_mempool::mocks::MockSharedMempool;
use cedra_storage_interface::mock::MockDbReaderWriter;
use cedra_types::chain_id::ChainId;
use std::sync::Arc;

// This is necessary for building the API with how the code is structured currently.
pub fn get_fake_context() -> Context {
    let mempool = MockSharedMempool::new_with_runtime();
    Context::new(
        ChainId::test(),
        Arc::new(MockDbReaderWriter),
        mempool.ac_client,
        NodeConfig::default(),
        None, /* table info reader */
    )
}
