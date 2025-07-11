// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of nodes without the
//! genesis.blob.

use crate::{
    config::{
        DiscoveryMethod, NetworkConfig, NodeConfig, Peer, PeerRole, PeerSet, HANDSHAKE_VERSION,
    },
    network_id::NetworkId,
};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};

pub struct ValidatorSwarm {
    pub nodes: Vec<NodeConfig>,
}

pub fn validator_swarm(
    template: &NodeConfig,
    count: usize,
    seed: [u8; 32],
    randomize_ports: bool,
) -> ValidatorSwarm {
    let mut rng = StdRng::from_seed(seed);
    let mut nodes = Vec::new();

    for _ in 0..count {
        let mut node = NodeConfig::generate_random_config_with_template(template, &mut rng);
        if randomize_ports {
            node.randomize_ports();
        }

        // For a validator node, any of its validator peers are considered an upstream peer
        let network = node.validator_network.as_mut().unwrap();
        network.discovery_method = DiscoveryMethod::Onchain;
        network.mutual_authentication = true;
        network.network_id = NetworkId::Validator;

        nodes.push(node);
    }

    // set the first validator as every validators' initial configured seed peer.
    let seed_config = &nodes[0].validator_network.as_ref().unwrap();
    let seeds = build_seed_for_network(seed_config, PeerRole::Validator);
    for node in &mut nodes {
        let network = node.validator_network.as_mut().unwrap();
        network.seeds.clone_from(&seeds);
    }

    nodes.sort_by(|a, b| {
        let a_addr = a.consensus.safety_rules.test.as_ref().unwrap().author;
        let b_addr = b.consensus.safety_rules.test.as_ref().unwrap().author;
        a_addr.cmp(&b_addr)
    });

    ValidatorSwarm { nodes }
}

pub fn validator_swarm_for_testing(nodes: usize) -> ValidatorSwarm {
    let config = NodeConfig::default();
    validator_swarm(&config, nodes, [1u8; 32], true)
}

/// Convenience function that builds a `PeerSet` containing a single peer for testing
/// with a fully formatted `NetworkAddress` containing its network identity pubkey
/// and handshake protocol version.
pub fn build_seed_for_network(seed_config: &NetworkConfig, seed_role: PeerRole) -> PeerSet {
    let seed_pubkey = cedra_crypto::PrivateKey::public_key(&seed_config.identity_key());
    let seed_addr = seed_config
        .listen_address
        .clone()
        .append_prod_protos(seed_pubkey, HANDSHAKE_VERSION);

    let mut keys = HashSet::new();
    keys.insert(seed_pubkey);
    let mut seeds = HashMap::default();
    seeds.insert(
        seed_config.peer_id(),
        Peer::new(vec![seed_addr], keys, seed_role),
    );
    seeds
}
