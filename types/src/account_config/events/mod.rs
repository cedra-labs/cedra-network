// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

pub mod burn;
pub mod burn_event;
pub mod burn_token;
pub mod burn_token_event;
pub mod cancel_offer;
pub mod claim;
pub mod coin_deposit;
pub mod coin_register;
pub mod coin_register_event;
pub mod coin_withdraw;
pub mod collection_description_mutate;
pub mod collection_description_mutate_event;
pub mod collection_maximum_mutate;
pub mod collection_maximum_mutate_event;
pub mod collection_mutation;
pub mod collection_mutation_event;
pub mod collection_uri_mutate;
pub mod collection_uri_mutate_event;
pub mod create_collection;
pub mod create_collection_event;
pub mod create_token_data_event;
pub mod default_property_mutate;
pub mod default_property_mutate_event;
pub mod deposit_event;
pub mod description_mutate;
pub mod description_mutate_event;
pub mod fungible_asset;
pub mod key_rotation;
pub mod key_rotation_event;
pub mod maximum_mutate;
pub mod maximum_mutate_event;
pub mod mint;
pub mod mint_event;
pub mod mint_token;
pub mod mint_token_event;
pub mod mutate_property_map;
pub mod mutate_token_property_map_event;
pub mod new_block;
pub mod new_epoch;
pub mod offer;
pub mod opt_in_transfer;
pub mod opt_in_transfer_event;
pub mod royalty_mutate;
pub mod royalty_mutate_event;
pub mod token_cancel_offer_event;
pub mod token_claim_event;
pub mod token_data_creation;
pub mod token_deposit;
pub mod token_deposit_event;
pub mod token_mutation;
pub mod token_mutation_event;
pub mod token_offer_event;
pub mod token_withdraw;
pub mod token_withdraw_event;
pub mod transfer;
pub mod transfer_event;
pub mod uri_mutation;
pub mod uri_mutation_event;
pub mod withdraw_event;

pub use burn::*;
pub use burn_event::*;
pub use burn_token::*;
pub use burn_token_event::*;
pub use cancel_offer::*;
pub use claim::*;
pub use coin_deposit::*;
pub use coin_register::*;
pub use coin_register_event::*;
pub use coin_withdraw::*;
pub use collection_description_mutate::*;
pub use collection_description_mutate_event::*;
pub use collection_maximum_mutate::*;
pub use collection_maximum_mutate_event::*;
pub use collection_mutation::*;
pub use collection_mutation_event::*;
pub use collection_uri_mutate::*;
pub use collection_uri_mutate_event::*;
pub use create_collection::*;
pub use create_collection_event::*;
pub use create_token_data_event::*;
pub use default_property_mutate::*;
pub use default_property_mutate_event::*;
pub use deposit_event::*;
pub use description_mutate::*;
pub use description_mutate_event::*;
pub use fungible_asset::*;
pub use key_rotation::*;
pub use key_rotation_event::*;
pub use maximum_mutate::*;
pub use maximum_mutate_event::*;
pub use mint::*;
pub use mint_event::*;
pub use mint_token::*;
pub use mint_token_event::*;
pub use mutate_property_map::*;
pub use mutate_token_property_map_event::*;
pub use new_block::*;
pub use new_epoch::*;
pub use offer::*;
pub use opt_in_transfer::*;
pub use opt_in_transfer_event::*;
pub use royalty_mutate::*;
pub use royalty_mutate_event::*;
pub use token_cancel_offer_event::*;
pub use token_claim_event::*;
pub use token_data_creation::*;
pub use token_deposit::*;
pub use token_deposit_event::*;
pub use token_mutation::*;
pub use token_mutation_event::*;
pub use token_offer_event::*;
pub use token_withdraw::*;
pub use token_withdraw_event::*;
pub use transfer::*;
pub use transfer_event::*;
pub use uri_mutation::*;
pub use uri_mutation_event::*;
pub use withdraw_event::*;

pub fn is_cedra_governance_create_proposal_event(event_type: &str) -> bool {
    event_type == "0x1::cedra_governance::CreateProposal"
        || event_type == "0x1::cedra_governance::CreateProposalEvent"
}
