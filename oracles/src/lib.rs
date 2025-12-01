// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

pub mod fetcher;
pub mod whitelist;
pub mod counters;

use std::sync::Arc;
use tokio::{runtime::Runtime};

use fetcher::PricesFetcher;
use whitelist::Whitelist;

use cedra_storage_interface::DbReader;
use cedra_types::{indexer::indexer_db_reader::IndexerReader};
use cedra_validator_transaction_pool::VTxnPoolState;

pub fn start_oracles_runtime(
    vtxn_pool: VTxnPoolState,
    db_reader: Arc<dyn DbReader>,
    indexer_reader: Option<Arc<dyn IndexerReader>>,
) -> Runtime {

//     let whitelist = Whitelist::new(db_reader, indexer_reader);
    let runtime = cedra_runtimes::spawn_named_runtime("oracles".into(), Some(4));

//     let prices_fetcher = PricesFetcher::new(
//         whitelist,
//         vtxn_pool,
//     );

//     runtime.spawn(async move {
//     if let Err(e) = prices_fetcher.fetch().await {
//         eprintln!("Price fetch failed: {:?}", e);
//     }
// });
    
    runtime
}

