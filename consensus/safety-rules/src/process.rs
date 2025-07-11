// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_safety_storage::PersistentSafetyStorage,
    remote_service::{self, RemoteService},
    safety_rules_manager,
};
use cedra_config::config::{SafetyRulesConfig, SafetyRulesService};
use std::net::SocketAddr;

pub struct Process {
    data: Option<ProcessData>,
}

impl Process {
    pub fn new(config: SafetyRulesConfig) -> Self {
        let storage = safety_rules_manager::storage(&config);

        let service = match &config.service {
            SafetyRulesService::Process(service) => service,
            _ => panic!("Unexpected SafetyRules service: {:?}", config.service),
        };
        let server_addr = service.server_address();

        Self {
            data: Some(ProcessData {
                server_addr,
                storage,
                network_timeout: config.network_timeout_ms,
            }),
        }
    }

    pub fn start(&mut self) {
        let data = self.data.take().expect("Unable to retrieve ProcessData");
        remote_service::execute(data.storage, data.server_addr, data.network_timeout);
    }
}

struct ProcessData {
    server_addr: SocketAddr,
    storage: PersistentSafetyStorage,
    // Timeout in Seconds for network operations
    network_timeout: u64,
}

pub struct ProcessService {
    server_addr: SocketAddr,
    network_timeout_ms: u64,
}

impl ProcessService {
    pub fn new(server_addr: SocketAddr, network_timeout: u64) -> Self {
        Self {
            server_addr,
            network_timeout_ms: network_timeout,
        }
    }
}

impl RemoteService for ProcessService {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }

    fn network_timeout_ms(&self) -> u64 {
        self.network_timeout_ms
    }
}
