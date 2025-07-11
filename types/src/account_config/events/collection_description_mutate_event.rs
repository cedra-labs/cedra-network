// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::move_utils::move_event_v1::MoveEventV1Type;
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
pub struct CollectionDescriptionMutateEvent {
    creator_addr: AccountAddress,
    collection_name: String,
    old_description: String,
    new_description: String,
}

impl CollectionDescriptionMutateEvent {
    pub fn new(
        creator_addr: AccountAddress,
        collection_name: String,
        old_description: String,
        new_description: String,
    ) -> Self {
        Self {
            creator_addr,
            collection_name,
            old_description,
            new_description,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn creator_addr(&self) -> &AccountAddress {
        &self.creator_addr
    }

    pub fn collection_name(&self) -> &String {
        &self.collection_name
    }

    pub fn old_description(&self) -> &String {
        &self.old_description
    }

    pub fn new_description(&self) -> &String {
        &self.new_description
    }
}

impl MoveStructType for CollectionDescriptionMutateEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("token_event_store");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CollectionDescriptionMutateEvent");
}

impl MoveEventV1Type for CollectionDescriptionMutateEvent {}

pub static COLLECTION_DESCRIPTION_MUTATE_EVENT_TYPE: Lazy<TypeTag> = Lazy::new(|| {
    TypeTag::Struct(Box::new(StructTag {
        address: TOKEN_ADDRESS,
        module: ident_str!("token_event_store").to_owned(),
        name: ident_str!("CollectionDescriptionMutateEvent").to_owned(),
        type_args: vec![],
    }))
});
