// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use crate::network_address::NetworkAddress;
use crate::{account_address::AccountAddress, validator_config::ValidatorConfig};
use cedra_crypto::bls12381;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt;

/// After executing a special transaction indicates a change to the next epoch, consensus
/// and networking get the new list of validators, their keys, and their voting power.  Consensus
/// has a public key to validate signed messages and networking will has public identity
/// keys for creating secure channels of communication between validators.  The validators and
/// their public keys and voting power may or may not change between epochs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorInfo {
    // The validator's account address. AccountAddresses are initially derived from the account
    // auth pubkey; however, the auth key can be rotated, so one should not rely on this
    // initial property.
    pub account_address: AccountAddress,
    // Voting power of this validator
    consensus_voting_power: u64,
    // Validator config
    config: ValidatorConfig,
}

impl fmt::Display for ValidatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "account_address: {}",
            self.account_address.short_str_lossless()
        )
    }
}

impl ValidatorInfo {
    pub fn new(
        account_address: AccountAddress,
        consensus_voting_power: u64,
        config: ValidatorConfig,
    ) -> Self {
        ValidatorInfo {
            account_address,
            consensus_voting_power,
            config,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_with_test_network_keys(
        account_address: AccountAddress,
        consensus_public_key: bls12381::PublicKey,
        consensus_voting_power: u64,
        validator_index: u64,
    ) -> Self {
        let addr = NetworkAddress::mock();
        let config = ValidatorConfig::new(
            consensus_public_key,
            bcs::to_bytes(&vec![addr.clone()]).unwrap(),
            bcs::to_bytes(&vec![addr]).unwrap(),
            validator_index,
        );

        Self {
            account_address,
            consensus_voting_power,
            config,
        }
    }

    /// Returns the id of this validator (hash of the current public key of the
    /// validator associated account address)
    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }

    /// Returns the key for validating signed messages from this validator
    pub fn consensus_public_key(&self) -> &bls12381::PublicKey {
        &self.config.consensus_public_key
    }

    /// Returns the voting power for this validator
    pub fn consensus_voting_power(&self) -> u64 {
        self.consensus_voting_power
    }

    /// Returns the validator's config
    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    /// Returns the validator's config, consuming self
    pub fn into_config(self) -> ValidatorConfig {
        self.config
    }
}
