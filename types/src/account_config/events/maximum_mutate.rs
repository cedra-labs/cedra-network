// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::move_utils::move_event_v2::MoveEventV2Type;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag, TOKEN_ADDRESS},
    move_resource::MoveStructType,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MaximumMutate {
    creator: AccountAddress,
    collection: String,
    token: String,
    old_maximum: u64,
    new_maximum: u64,
}

impl MaximumMutate {
    pub fn new(
        creator: AccountAddress,
        collection: String,
        token: String,
        old_maximum: u64,
        new_maximum: u64,
    ) -> Self {
        Self {
            creator,
            collection,
            token,
            old_maximum,
            new_maximum,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn creator(&self) -> &AccountAddress {
        &self.creator
    }

    pub fn collection(&self) -> &String {
        &self.collection
    }

    pub fn token(&self) -> &String {
        &self.token
    }

    pub fn old_maximum(&self) -> &u64 {
        &self.old_maximum
    }

    pub fn new_maximum(&self) -> &u64 {
        &self.new_maximum
    }
}

impl MoveStructType for MaximumMutate {
    const MODULE_NAME: &'static IdentStr = ident_str!("token_event_store");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MaximumMutate");
}

impl MoveEventV2Type for MaximumMutate {}

pub static MAXIMUM_MUTATE_TYPE: Lazy<TypeTag> = Lazy::new(|| {
    TypeTag::Struct(Box::new(StructTag {
        address: TOKEN_ADDRESS,
        module: ident_str!("token_event_store").to_owned(),
        name: ident_str!("MaximumMutate").to_owned(),
        type_args: vec![],
    }))
});
