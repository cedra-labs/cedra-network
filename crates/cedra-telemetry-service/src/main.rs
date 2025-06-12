#![forbid(unsafe_code)]

// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_telemetry_service::CedraTelemetryServiceArgs;
use clap::Parser;

#[tokio::main]
async fn main() {
    cedra_logger::Logger::new().init();
    CedraTelemetryServiceArgs::parse().run().await;
}
