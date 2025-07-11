// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

// This file was generated. Do not modify!
//
// To update this code, run: `cargo run --release -p framework`.

// Conversion library between a structured representation of a Move script call (`ScriptCall`) and the
// standard BCS-compatible representation used in Cedra transactions (`Script`).
//
// This code was generated by compiling known Script interfaces ("ABIs") with the tool `cedra-sdk-builder`.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::get_first)]
use cedra_types::{
    account_address::AccountAddress,
    transaction::{EntryFunction, TransactionPayload},
};
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, TypeTag},
};

type Bytes = Vec<u8>;

/// Structured representation of a call into a known Move entry function.
/// ```ignore
/// impl EntryFunctionCall {
///     pub fn encode(self) -> TransactionPayload { .. }
///     pub fn decode(&TransactionPayload) -> Option<EntryFunctionCall> { .. }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "fuzzing", proptest(no_params))]
pub enum EntryFunctionCall {
    /// Burn a token by the token owner
    TokenBurn {
        creators_address: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        property_version: u64,
        amount: u64,
    },

    /// Burn a token by creator when the token's BURNABLE_BY_CREATOR is true
    /// The token is owned at address owner
    TokenBurnByCreator {
        owner: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        property_version: u64,
        amount: u64,
    },

    /// create a empty token collection with parameters
    TokenCreateCollectionScript {
        name: Vec<u8>,
        description: Vec<u8>,
        uri: Vec<u8>,
        maximum: u64,
        mutate_setting: Vec<bool>,
    },

    /// create token with raw inputs
    TokenCreateTokenScript {
        collection: Vec<u8>,
        name: Vec<u8>,
        description: Vec<u8>,
        balance: u64,
        maximum: u64,
        uri: Vec<u8>,
        royalty_payee_address: AccountAddress,
        royalty_points_denominator: u64,
        royalty_points_numerator: u64,
        mutate_setting: Vec<bool>,
        property_keys: Vec<Vec<u8>>,
        property_values: Vec<Vec<u8>>,
        property_types: Vec<Vec<u8>>,
    },

    TokenDirectTransferScript {
        creators_address: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        property_version: u64,
        amount: u64,
    },

    TokenInitializeTokenScript {},

    /// Mint more token from an existing token_data. Mint only adds more token to property_version 0
    TokenMintScript {
        token_data_address: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        amount: u64,
    },

    /// mutate the token property and save the new property in TokenStore
    /// if the token property_version is 0, we will create a new property_version per token to generate a new token_id per token
    /// if the token property_version is not 0, we will just update the propertyMap and use the existing token_id (property_version)
    TokenMutateTokenProperties {
        token_owner: AccountAddress,
        creator: AccountAddress,
        collection_name: Vec<u8>,
        token_name: Vec<u8>,
        token_property_version: u64,
        amount: u64,
        keys: Vec<Vec<u8>>,
        values: Vec<Vec<u8>>,
        types: Vec<Vec<u8>>,
    },

    TokenOptInDirectTransfer {
        opt_in: bool,
    },

    /// Transfers `amount` of tokens from `from` to `to`.
    /// The receiver `to` has to opt-in direct transfer first
    TokenTransferWithOptIn {
        creator: AccountAddress,
        collection_name: Vec<u8>,
        token_name: Vec<u8>,
        token_property_version: u64,
        to: AccountAddress,
        amount: u64,
    },

    /// Token owner lists their token for swapping
    TokenCoinSwapListTokenForSwap {
        coin_type: TypeTag,
        _creators_address: AccountAddress,
        _collection: Vec<u8>,
        _name: Vec<u8>,
        _property_version: u64,
        _token_amount: u64,
        _min_coin_per_token: u64,
        _locked_until_secs: u64,
    },

    TokenTransfersCancelOfferScript {
        receiver: AccountAddress,
        creator: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        property_version: u64,
    },

    TokenTransfersClaimScript {
        sender: AccountAddress,
        creator: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        property_version: u64,
    },

    TokenTransfersOfferScript {
        receiver: AccountAddress,
        creator: AccountAddress,
        collection: Vec<u8>,
        name: Vec<u8>,
        property_version: u64,
        amount: u64,
    },
}

impl EntryFunctionCall {
    /// Build an Cedra `TransactionPayload` from a structured object `EntryFunctionCall`.
    pub fn encode(self) -> TransactionPayload {
        use EntryFunctionCall::*;
        match self {
            TokenBurn {
                creators_address,
                collection,
                name,
                property_version,
                amount,
            } => token_burn(creators_address, collection, name, property_version, amount),
            TokenBurnByCreator {
                owner,
                collection,
                name,
                property_version,
                amount,
            } => token_burn_by_creator(owner, collection, name, property_version, amount),
            TokenCreateCollectionScript {
                name,
                description,
                uri,
                maximum,
                mutate_setting,
            } => token_create_collection_script(name, description, uri, maximum, mutate_setting),
            TokenCreateTokenScript {
                collection,
                name,
                description,
                balance,
                maximum,
                uri,
                royalty_payee_address,
                royalty_points_denominator,
                royalty_points_numerator,
                mutate_setting,
                property_keys,
                property_values,
                property_types,
            } => token_create_token_script(
                collection,
                name,
                description,
                balance,
                maximum,
                uri,
                royalty_payee_address,
                royalty_points_denominator,
                royalty_points_numerator,
                mutate_setting,
                property_keys,
                property_values,
                property_types,
            ),
            TokenDirectTransferScript {
                creators_address,
                collection,
                name,
                property_version,
                amount,
            } => token_direct_transfer_script(
                creators_address,
                collection,
                name,
                property_version,
                amount,
            ),
            TokenInitializeTokenScript {} => token_initialize_token_script(),
            TokenMintScript {
                token_data_address,
                collection,
                name,
                amount,
            } => token_mint_script(token_data_address, collection, name, amount),
            TokenMutateTokenProperties {
                token_owner,
                creator,
                collection_name,
                token_name,
                token_property_version,
                amount,
                keys,
                values,
                types,
            } => token_mutate_token_properties(
                token_owner,
                creator,
                collection_name,
                token_name,
                token_property_version,
                amount,
                keys,
                values,
                types,
            ),
            TokenOptInDirectTransfer { opt_in } => token_opt_in_direct_transfer(opt_in),
            TokenTransferWithOptIn {
                creator,
                collection_name,
                token_name,
                token_property_version,
                to,
                amount,
            } => token_transfer_with_opt_in(
                creator,
                collection_name,
                token_name,
                token_property_version,
                to,
                amount,
            ),
            TokenCoinSwapListTokenForSwap {
                coin_type,
                _creators_address,
                _collection,
                _name,
                _property_version,
                _token_amount,
                _min_coin_per_token,
                _locked_until_secs,
            } => token_coin_swap_list_token_for_swap(
                coin_type,
                _creators_address,
                _collection,
                _name,
                _property_version,
                _token_amount,
                _min_coin_per_token,
                _locked_until_secs,
            ),
            TokenTransfersCancelOfferScript {
                receiver,
                creator,
                collection,
                name,
                property_version,
            } => token_transfers_cancel_offer_script(
                receiver,
                creator,
                collection,
                name,
                property_version,
            ),
            TokenTransfersClaimScript {
                sender,
                creator,
                collection,
                name,
                property_version,
            } => token_transfers_claim_script(sender, creator, collection, name, property_version),
            TokenTransfersOfferScript {
                receiver,
                creator,
                collection,
                name,
                property_version,
                amount,
            } => token_transfers_offer_script(
                receiver,
                creator,
                collection,
                name,
                property_version,
                amount,
            ),
        }
    }

    /// Try to recognize an Cedra `TransactionPayload` and convert it into a structured object `EntryFunctionCall`.
    pub fn decode(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            match SCRIPT_FUNCTION_DECODER_MAP.get(&format!(
                "{}_{}",
                script.module().name(),
                script.function()
            )) {
                Some(decoder) => decoder(payload),
                None => None,
            }
        } else {
            None
        }
    }
}

/// Burn a token by the token owner
pub fn token_burn(
    creators_address: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    property_version: u64,
    amount: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("burn").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&creators_address).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&property_version).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
        ],
    ))
}

/// Burn a token by creator when the token's BURNABLE_BY_CREATOR is true
/// The token is owned at address owner
pub fn token_burn_by_creator(
    owner: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    property_version: u64,
    amount: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("burn_by_creator").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&owner).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&property_version).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
        ],
    ))
}

/// create a empty token collection with parameters
pub fn token_create_collection_script(
    name: Vec<u8>,
    description: Vec<u8>,
    uri: Vec<u8>,
    maximum: u64,
    mutate_setting: Vec<bool>,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("create_collection_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&description).unwrap(),
            bcs::to_bytes(&uri).unwrap(),
            bcs::to_bytes(&maximum).unwrap(),
            bcs::to_bytes(&mutate_setting).unwrap(),
        ],
    ))
}

/// create token with raw inputs
pub fn token_create_token_script(
    collection: Vec<u8>,
    name: Vec<u8>,
    description: Vec<u8>,
    balance: u64,
    maximum: u64,
    uri: Vec<u8>,
    royalty_payee_address: AccountAddress,
    royalty_points_denominator: u64,
    royalty_points_numerator: u64,
    mutate_setting: Vec<bool>,
    property_keys: Vec<Vec<u8>>,
    property_values: Vec<Vec<u8>>,
    property_types: Vec<Vec<u8>>,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("create_token_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&description).unwrap(),
            bcs::to_bytes(&balance).unwrap(),
            bcs::to_bytes(&maximum).unwrap(),
            bcs::to_bytes(&uri).unwrap(),
            bcs::to_bytes(&royalty_payee_address).unwrap(),
            bcs::to_bytes(&royalty_points_denominator).unwrap(),
            bcs::to_bytes(&royalty_points_numerator).unwrap(),
            bcs::to_bytes(&mutate_setting).unwrap(),
            bcs::to_bytes(&property_keys).unwrap(),
            bcs::to_bytes(&property_values).unwrap(),
            bcs::to_bytes(&property_types).unwrap(),
        ],
    ))
}

pub fn token_direct_transfer_script(
    creators_address: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    property_version: u64,
    amount: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("direct_transfer_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&creators_address).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&property_version).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
        ],
    ))
}

pub fn token_initialize_token_script() -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("initialize_token_script").to_owned(),
        vec![],
        vec![],
    ))
}

/// Mint more token from an existing token_data. Mint only adds more token to property_version 0
pub fn token_mint_script(
    token_data_address: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    amount: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("mint_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&token_data_address).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
        ],
    ))
}

/// mutate the token property and save the new property in TokenStore
/// if the token property_version is 0, we will create a new property_version per token to generate a new token_id per token
/// if the token property_version is not 0, we will just update the propertyMap and use the existing token_id (property_version)
pub fn token_mutate_token_properties(
    token_owner: AccountAddress,
    creator: AccountAddress,
    collection_name: Vec<u8>,
    token_name: Vec<u8>,
    token_property_version: u64,
    amount: u64,
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    types: Vec<Vec<u8>>,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("mutate_token_properties").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&token_owner).unwrap(),
            bcs::to_bytes(&creator).unwrap(),
            bcs::to_bytes(&collection_name).unwrap(),
            bcs::to_bytes(&token_name).unwrap(),
            bcs::to_bytes(&token_property_version).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
            bcs::to_bytes(&keys).unwrap(),
            bcs::to_bytes(&values).unwrap(),
            bcs::to_bytes(&types).unwrap(),
        ],
    ))
}

pub fn token_opt_in_direct_transfer(opt_in: bool) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("opt_in_direct_transfer").to_owned(),
        vec![],
        vec![bcs::to_bytes(&opt_in).unwrap()],
    ))
}

/// Transfers `amount` of tokens from `from` to `to`.
/// The receiver `to` has to opt-in direct transfer first
pub fn token_transfer_with_opt_in(
    creator: AccountAddress,
    collection_name: Vec<u8>,
    token_name: Vec<u8>,
    token_property_version: u64,
    to: AccountAddress,
    amount: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token").to_owned(),
        ),
        ident_str!("transfer_with_opt_in").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&creator).unwrap(),
            bcs::to_bytes(&collection_name).unwrap(),
            bcs::to_bytes(&token_name).unwrap(),
            bcs::to_bytes(&token_property_version).unwrap(),
            bcs::to_bytes(&to).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
        ],
    ))
}

/// Token owner lists their token for swapping
pub fn token_coin_swap_list_token_for_swap(
    coin_type: TypeTag,
    _creators_address: AccountAddress,
    _collection: Vec<u8>,
    _name: Vec<u8>,
    _property_version: u64,
    _token_amount: u64,
    _min_coin_per_token: u64,
    _locked_until_secs: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token_coin_swap").to_owned(),
        ),
        ident_str!("list_token_for_swap").to_owned(),
        vec![coin_type],
        vec![
            bcs::to_bytes(&_creators_address).unwrap(),
            bcs::to_bytes(&_collection).unwrap(),
            bcs::to_bytes(&_name).unwrap(),
            bcs::to_bytes(&_property_version).unwrap(),
            bcs::to_bytes(&_token_amount).unwrap(),
            bcs::to_bytes(&_min_coin_per_token).unwrap(),
            bcs::to_bytes(&_locked_until_secs).unwrap(),
        ],
    ))
}

pub fn token_transfers_cancel_offer_script(
    receiver: AccountAddress,
    creator: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    property_version: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token_transfers").to_owned(),
        ),
        ident_str!("cancel_offer_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&receiver).unwrap(),
            bcs::to_bytes(&creator).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&property_version).unwrap(),
        ],
    ))
}

pub fn token_transfers_claim_script(
    sender: AccountAddress,
    creator: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    property_version: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token_transfers").to_owned(),
        ),
        ident_str!("claim_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&sender).unwrap(),
            bcs::to_bytes(&creator).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&property_version).unwrap(),
        ],
    ))
}

pub fn token_transfers_offer_script(
    receiver: AccountAddress,
    creator: AccountAddress,
    collection: Vec<u8>,
    name: Vec<u8>,
    property_version: u64,
    amount: u64,
) -> TransactionPayload {
    TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            AccountAddress::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3,
            ]),
            ident_str!("token_transfers").to_owned(),
        ),
        ident_str!("offer_script").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&receiver).unwrap(),
            bcs::to_bytes(&creator).unwrap(),
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&property_version).unwrap(),
            bcs::to_bytes(&amount).unwrap(),
        ],
    ))
}
mod decoder {
    use super::*;
    pub fn token_burn(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenBurn {
                creators_address: bcs::from_bytes(script.args().get(0)?).ok()?,
                collection: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                property_version: bcs::from_bytes(script.args().get(3)?).ok()?,
                amount: bcs::from_bytes(script.args().get(4)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_burn_by_creator(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenBurnByCreator {
                owner: bcs::from_bytes(script.args().get(0)?).ok()?,
                collection: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                property_version: bcs::from_bytes(script.args().get(3)?).ok()?,
                amount: bcs::from_bytes(script.args().get(4)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_create_collection_script(
        payload: &TransactionPayload,
    ) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenCreateCollectionScript {
                name: bcs::from_bytes(script.args().get(0)?).ok()?,
                description: bcs::from_bytes(script.args().get(1)?).ok()?,
                uri: bcs::from_bytes(script.args().get(2)?).ok()?,
                maximum: bcs::from_bytes(script.args().get(3)?).ok()?,
                mutate_setting: bcs::from_bytes(script.args().get(4)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_create_token_script(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenCreateTokenScript {
                collection: bcs::from_bytes(script.args().get(0)?).ok()?,
                name: bcs::from_bytes(script.args().get(1)?).ok()?,
                description: bcs::from_bytes(script.args().get(2)?).ok()?,
                balance: bcs::from_bytes(script.args().get(3)?).ok()?,
                maximum: bcs::from_bytes(script.args().get(4)?).ok()?,
                uri: bcs::from_bytes(script.args().get(5)?).ok()?,
                royalty_payee_address: bcs::from_bytes(script.args().get(6)?).ok()?,
                royalty_points_denominator: bcs::from_bytes(script.args().get(7)?).ok()?,
                royalty_points_numerator: bcs::from_bytes(script.args().get(8)?).ok()?,
                mutate_setting: bcs::from_bytes(script.args().get(9)?).ok()?,
                property_keys: bcs::from_bytes(script.args().get(10)?).ok()?,
                property_values: bcs::from_bytes(script.args().get(11)?).ok()?,
                property_types: bcs::from_bytes(script.args().get(12)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_direct_transfer_script(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenDirectTransferScript {
                creators_address: bcs::from_bytes(script.args().get(0)?).ok()?,
                collection: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                property_version: bcs::from_bytes(script.args().get(3)?).ok()?,
                amount: bcs::from_bytes(script.args().get(4)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_initialize_token_script(
        payload: &TransactionPayload,
    ) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(_script) = payload {
            Some(EntryFunctionCall::TokenInitializeTokenScript {})
        } else {
            None
        }
    }

    pub fn token_mint_script(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenMintScript {
                token_data_address: bcs::from_bytes(script.args().get(0)?).ok()?,
                collection: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                amount: bcs::from_bytes(script.args().get(3)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_mutate_token_properties(
        payload: &TransactionPayload,
    ) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenMutateTokenProperties {
                token_owner: bcs::from_bytes(script.args().get(0)?).ok()?,
                creator: bcs::from_bytes(script.args().get(1)?).ok()?,
                collection_name: bcs::from_bytes(script.args().get(2)?).ok()?,
                token_name: bcs::from_bytes(script.args().get(3)?).ok()?,
                token_property_version: bcs::from_bytes(script.args().get(4)?).ok()?,
                amount: bcs::from_bytes(script.args().get(5)?).ok()?,
                keys: bcs::from_bytes(script.args().get(6)?).ok()?,
                values: bcs::from_bytes(script.args().get(7)?).ok()?,
                types: bcs::from_bytes(script.args().get(8)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_opt_in_direct_transfer(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenOptInDirectTransfer {
                opt_in: bcs::from_bytes(script.args().get(0)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_transfer_with_opt_in(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenTransferWithOptIn {
                creator: bcs::from_bytes(script.args().get(0)?).ok()?,
                collection_name: bcs::from_bytes(script.args().get(1)?).ok()?,
                token_name: bcs::from_bytes(script.args().get(2)?).ok()?,
                token_property_version: bcs::from_bytes(script.args().get(3)?).ok()?,
                to: bcs::from_bytes(script.args().get(4)?).ok()?,
                amount: bcs::from_bytes(script.args().get(5)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_coin_swap_list_token_for_swap(
        payload: &TransactionPayload,
    ) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenCoinSwapListTokenForSwap {
                coin_type: script.ty_args().get(0)?.clone(),
                _creators_address: bcs::from_bytes(script.args().get(0)?).ok()?,
                _collection: bcs::from_bytes(script.args().get(1)?).ok()?,
                _name: bcs::from_bytes(script.args().get(2)?).ok()?,
                _property_version: bcs::from_bytes(script.args().get(3)?).ok()?,
                _token_amount: bcs::from_bytes(script.args().get(4)?).ok()?,
                _min_coin_per_token: bcs::from_bytes(script.args().get(5)?).ok()?,
                _locked_until_secs: bcs::from_bytes(script.args().get(6)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_transfers_cancel_offer_script(
        payload: &TransactionPayload,
    ) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenTransfersCancelOfferScript {
                receiver: bcs::from_bytes(script.args().get(0)?).ok()?,
                creator: bcs::from_bytes(script.args().get(1)?).ok()?,
                collection: bcs::from_bytes(script.args().get(2)?).ok()?,
                name: bcs::from_bytes(script.args().get(3)?).ok()?,
                property_version: bcs::from_bytes(script.args().get(4)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_transfers_claim_script(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenTransfersClaimScript {
                sender: bcs::from_bytes(script.args().get(0)?).ok()?,
                creator: bcs::from_bytes(script.args().get(1)?).ok()?,
                collection: bcs::from_bytes(script.args().get(2)?).ok()?,
                name: bcs::from_bytes(script.args().get(3)?).ok()?,
                property_version: bcs::from_bytes(script.args().get(4)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn token_transfers_offer_script(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::EntryFunction(script) = payload {
            Some(EntryFunctionCall::TokenTransfersOfferScript {
                receiver: bcs::from_bytes(script.args().get(0)?).ok()?,
                creator: bcs::from_bytes(script.args().get(1)?).ok()?,
                collection: bcs::from_bytes(script.args().get(2)?).ok()?,
                name: bcs::from_bytes(script.args().get(3)?).ok()?,
                property_version: bcs::from_bytes(script.args().get(4)?).ok()?,
                amount: bcs::from_bytes(script.args().get(5)?).ok()?,
            })
        } else {
            None
        }
    }
}

type EntryFunctionDecoderMap = std::collections::HashMap<
    String,
    Box<
        dyn Fn(&TransactionPayload) -> Option<EntryFunctionCall>
            + std::marker::Sync
            + std::marker::Send,
    >,
>;

static SCRIPT_FUNCTION_DECODER_MAP: once_cell::sync::Lazy<EntryFunctionDecoderMap> =
    once_cell::sync::Lazy::new(|| {
        let mut map: EntryFunctionDecoderMap = std::collections::HashMap::new();
        map.insert("token_burn".to_string(), Box::new(decoder::token_burn));
        map.insert(
            "token_burn_by_creator".to_string(),
            Box::new(decoder::token_burn_by_creator),
        );
        map.insert(
            "token_create_collection_script".to_string(),
            Box::new(decoder::token_create_collection_script),
        );
        map.insert(
            "token_create_token_script".to_string(),
            Box::new(decoder::token_create_token_script),
        );
        map.insert(
            "token_direct_transfer_script".to_string(),
            Box::new(decoder::token_direct_transfer_script),
        );
        map.insert(
            "token_initialize_token_script".to_string(),
            Box::new(decoder::token_initialize_token_script),
        );
        map.insert(
            "token_mint_script".to_string(),
            Box::new(decoder::token_mint_script),
        );
        map.insert(
            "token_mutate_token_properties".to_string(),
            Box::new(decoder::token_mutate_token_properties),
        );
        map.insert(
            "token_opt_in_direct_transfer".to_string(),
            Box::new(decoder::token_opt_in_direct_transfer),
        );
        map.insert(
            "token_transfer_with_opt_in".to_string(),
            Box::new(decoder::token_transfer_with_opt_in),
        );
        map.insert(
            "token_coin_swap_list_token_for_swap".to_string(),
            Box::new(decoder::token_coin_swap_list_token_for_swap),
        );
        map.insert(
            "token_transfers_cancel_offer_script".to_string(),
            Box::new(decoder::token_transfers_cancel_offer_script),
        );
        map.insert(
            "token_transfers_claim_script".to_string(),
            Box::new(decoder::token_transfers_claim_script),
        );
        map.insert(
            "token_transfers_offer_script".to_string(),
            Box::new(decoder::token_transfers_offer_script),
        );
        map
    });
