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
pub struct CollectionUriMutate {
    creator_addr: AccountAddress,
    collection_name: String,
    old_uri: String,
    new_uri: String,
}

impl CollectionUriMutate {
    pub fn new(
        creator_addr: AccountAddress,
        collection_name: String,
        old_uri: String,
        new_uri: String,
    ) -> Self {
        Self {
            creator_addr,
            collection_name,
            old_uri,
            new_uri,
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

    pub fn old_uri(&self) -> &String {
        &self.old_uri
    }

    pub fn new_uri(&self) -> &String {
        &self.new_uri
    }
}

impl MoveStructType for CollectionUriMutate {
    const MODULE_NAME: &'static IdentStr = ident_str!("token_event_store");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CollectionUriMutate");
}

impl MoveEventV2Type for CollectionUriMutate {}

pub static COLLECTION_URI_MUTATE_TYPE: Lazy<TypeTag> = Lazy::new(|| {
    TypeTag::Struct(Box::new(StructTag {
        address: TOKEN_ADDRESS,
        module: ident_str!("token_event_store").to_owned(),
        name: ident_str!("CollectionUriMutate").to_owned(),
        type_args: vec![],
    }))
});
