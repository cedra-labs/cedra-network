// Copyright (c) Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    cedra_workspace_server::WorkspaceCommand::parse()
        .run()
        .await
}
