// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

//! The Cedra API response / error handling philosophy.
//!
//! The return type for every endpoint should be a
//! `poem::Result<MyResponse<T>, MyError>` where MyResponse is an instance of
//! ApiResponse that contains only the status codes that it can actually
//! return. This will manifest in the OpenAPI spec, making it clear to users
//! what the API can actually return. The error should operate the same way,
//! where it only describes error codes that it can actually return.
//!
//! Every endpoint should be able to return data as JSON or BCS, depending on
//! the Accept header given by the client. If none is given, default to JSON.
//!
//! The client should be able to provide data to POST endpoints as either JSON
//! or BCS, given they provide the appropriate Content-Type header.
//!
//! Where possible, if the API is returning data as BCS, it should pull the
//! bytes directly from the DB where possible without further processing.
//!
//! Internally, there are many functions that can return errors. The context
//! for what type of error those functions return is lost if we return an
//! opaque error type like anyhow::Error. As such, it is important that each
//! function return error types that capture the intended status code. This
//! module defines traits to help with this, ensuring that the error types
//! returned by each function and its callers is enforced at compile time.
//! See generate_error_traits and its invocations for more on this topic.

// TODO: https://github.com/cedra-labs/cedra-network/issues/2279

use super::{accept_type::AcceptType, bcs_payload::Bcs};
use cedra_api_types::{Address, CedraError, CedraErrorCode, HashValue, LedgerInfo};
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
};
use poem_openapi::{payload::Json, types::ToJSON, ResponseContent};
use serde_json::Value;
use std::fmt::Display;

/// An enum representing the different types of outputs for APIs
#[derive(ResponseContent)]
pub enum CedraResponseContent<T: ToJSON + Send + Sync> {
    /// When returning data as JSON, we take in T and then serialize to JSON as
    /// part of the response.
    Json(Json<T>),

    /// Return the data as BCS, which is just Vec<u8>. This data could have come
    /// from either an internal Rust type being serialized into bytes, or just
    /// the bytes directly from storage.
    Bcs(Bcs),
}

/// This trait defines common functions that all error responses should impl.
/// As a user you shouldn't worry about this, the generate_error_response macro
/// takes care of it for you. Mostly these are helpers to allow callers holding
/// an error response to manipulate the CedraError inside it.
pub trait CedraErrorResponse {
    fn inner_mut(&mut self) -> &mut CedraError;
}

/// This macro defines traits for all of the given status codes. In each trait
/// there is a function that defines a helper for building an instance of the
/// error type using that code. These traits are helpful for defining what
/// error types an internal function can return. For example, the failpoint
/// function can return an internal error, so its signature would look like:
/// fn fail_point_poem<E: InternalError>(name: &str) -> anyhow::Result<(), E>
/// This should be invoked just once, taking in every status that we use
/// throughout the entire API. Every one of these traits requires that the
/// implementor also implements CedraErrorResponse, which saves functions from
/// having to add that bound to errors themselves.
#[macro_export]
macro_rules! generate_error_traits {
    ($($trait_name:ident),*) => {
        paste::paste! {
        $(
        pub trait [<$trait_name Error>]: CedraErrorResponse {
            // With ledger info and an error code
            #[allow(unused)]
            fn [<$trait_name:snake _with_code>]<Err: std::fmt::Display>(
                err: Err,
                error_code: cedra_api_types::CedraErrorCode,
                ledger_info: &cedra_api_types::LedgerInfo,
            ) -> Self where Self: Sized;

            // With an error code and no ledger info headers (special case)
            #[allow(unused)]
            fn [<$trait_name:snake _with_code_no_info>]<Err: std::fmt::Display>(
                err: Err,
                error_code: cedra_api_types::CedraErrorCode,
            ) -> Self where Self: Sized;

            #[allow(unused)]
            fn [<$trait_name:snake _with_vm_status>]<Err: std::fmt::Display>(
                err: Err,
                error_code: cedra_api_types::CedraErrorCode,
                vm_status: cedra_types::vm_status::StatusCode,
                ledger_info: &cedra_api_types::LedgerInfo
            ) -> Self where Self: Sized;

            #[allow(unused)]
            fn [<$trait_name:snake _from_cedra_error>](
                cedra_error: cedra_api_types::CedraError,
                ledger_info: &cedra_api_types::LedgerInfo
            ) -> Self where Self: Sized;
        }
        )*
        }
    };
}

/// This macro helps generate types, From impls, etc. for an error
/// response from a Poem endpoint. It generates a response type that only has
/// the specified response codes, which is then reflected in the OpenAPI spec.
/// For each status code given to a particular invocation of this macro, we
/// implement the relevant trait from generate_error_traits.
/// See the comments in the macro for an explanation of what is happening.
#[macro_export]
macro_rules! generate_error_response {
    ($enum_name:ident, $(($status:literal, $name:ident)),*) => {
        // We use the paste crate to allow us to generate the name of the code
        // enum, more on that in the comment above that enum.
        paste::paste! {

        // Generate an enum with name `enum_name`. Iterate through each of the
        // response codes, generating a variant for each with the given name
        // and status code. We always generate a variant for 500.
        #[derive(Debug, poem_openapi::ApiResponse)]
        pub enum $enum_name {
            $(
            #[oai(status = $status)]
            $name(poem_openapi::payload::Json<Box<cedra_api_types::CedraError>>,
                // We use just regular u64 here instead of U64 since all header
                // values are implicitly strings anyway.
                /// Chain ID of the current chain
                #[oai(header = "X-Cedra-Chain-Id")] Option<u8>,
                /// Current ledger version of the chain
                #[oai(header = "X-Cedra-Ledger-Version")] Option<u64>,
                /// Oldest non-pruned ledger version of the chain
                #[oai(header = "X-Cedra-Ledger-Oldest-Version")] Option<u64>,
                /// Current timestamp of the chain
                #[oai(header = "X-Cedra-Ledger-TimestampUsec")] Option<u64>,
                /// Current epoch of the chain
                #[oai(header = "X-Cedra-Epoch")] Option<u64>,
                /// Current block height of the chain
                #[oai(header = "X-Cedra-Block-Height")] Option<u64>,
                /// Oldest non-pruned block height of the chain
                #[oai(header = "X-Cedra-Oldest-Block-Height")] Option<u64>,
                /// The cost of the call in terms of gas
                #[oai(header = "X-Cedra-Gas-Used")] Option<u64>,
            ),
            )*
        }

        // For each status, implement the relevant error trait. This means if
        // the macro invocation specifies Internal and BadRequest, the
        // functions internal(anyhow::Error) and bad_request(anyhow::Error)
        // will be generated. There are also variants for taking in strs.
        $(
        impl $crate::response::[<$name Error>] for $enum_name {
            fn [<$name:snake _with_code>]<Err: std::fmt::Display>(
                err: Err,
                error_code: cedra_api_types::CedraErrorCode,
                ledger_info: &cedra_api_types::LedgerInfo
            )-> Self where Self: Sized {
                let error = cedra_api_types::CedraError::new_with_error_code(err, error_code);
                let payload = poem_openapi::payload::Json(Box::new(error));

                Self::from($enum_name::$name(
                    payload,
                    Some(ledger_info.chain_id),
                    Some(ledger_info.ledger_version.into()),
                    Some(ledger_info.oldest_ledger_version.into()),
                    Some(ledger_info.ledger_timestamp.into()),
                    Some(ledger_info.epoch.into()),
                    Some(ledger_info.block_height.into()),
                    Some(ledger_info.oldest_block_height.into()),
                    None,
                ))
            }

            fn [<$name:snake _with_code_no_info>]<Err: std::fmt::Display>(
                err: Err,
                error_code: cedra_api_types::CedraErrorCode,
            )-> Self where Self: Sized {
                let error = cedra_api_types::CedraError::new_with_error_code(err, error_code);
                let payload = poem_openapi::payload::Json(Box::new(error));

                Self::from($enum_name::$name(
                    payload,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ))
            }

            fn [<$name:snake _with_vm_status>]<Err: std::fmt::Display>(
                err: Err,
                error_code: cedra_api_types::CedraErrorCode,
                vm_status: cedra_types::vm_status::StatusCode,
                ledger_info: &cedra_api_types::LedgerInfo
            ) -> Self where Self: Sized {
                let error = cedra_api_types::CedraError::new_with_vm_status(err, error_code, vm_status);
                let payload = poem_openapi::payload::Json(Box::new(error));
                Self::from($enum_name::$name(
                    payload,
                    Some(ledger_info.chain_id),
                    Some(ledger_info.ledger_version.into()),
                    Some(ledger_info.oldest_ledger_version.into()),
                    Some(ledger_info.ledger_timestamp.into()),
                    Some(ledger_info.epoch.into()),
                    Some(ledger_info.block_height.into()),
                    Some(ledger_info.oldest_block_height.into()),
                    None,
                ))
            }

            fn [<$name:snake _from_cedra_error>](
                cedra_error: cedra_api_types::CedraError,
                ledger_info: &cedra_api_types::LedgerInfo
            ) -> Self where Self: Sized {
                let payload = poem_openapi::payload::Json(Box::new(cedra_error));
                Self::from($enum_name::$name(
                    payload,
                    Some(ledger_info.chain_id),
                    Some(ledger_info.ledger_version.into()),
                    Some(ledger_info.oldest_ledger_version.into()),
                    Some(ledger_info.ledger_timestamp.into()),
                    Some(ledger_info.epoch.into()),
                    Some(ledger_info.block_height.into()),
                    Some(ledger_info.oldest_block_height.into()),
                    None,
                ))
            }
        }
        )*
        }

        // Generate a function that helps get the CedraError within.
        impl $crate::response::CedraErrorResponse for $enum_name {
            fn inner_mut(&mut self) -> &mut cedra_api_types::CedraError {
                match self {
                    $(
                    $enum_name::$name(poem_openapi::payload::Json(inner),
                        _chain_id,
                        _ledger_version,
                        _oldest_ledger_version,
                        _ledger_timestamp,
                        _epoch,
                        _block_height,
                        _oldest_block_height,
                        _gas_used,
                    ) => &mut *inner,
                    )*
                }
            }
        }

        impl std::error::Error for $enum_name {}

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

/// This macro helps generate types, From impls, etc. for a successful response
/// from a Poem endpoint. It generates a response type that only has the
/// specified response codes, which is then reflected in the OpenAPI spec.
/// See the comments in the macro for an explanation of what is happening.
#[macro_export]
macro_rules! generate_success_response {
    ($enum_name:ident, $(($status:literal, $name:ident)),*) => {
        // We use the paste crate to allows us to generate the name of the code
        // enum, more on that in the comment above that enum.
        paste::paste! {

        // Generate an enum with name `enum_name`. Iterate through each of the
        // response codes, generating a variant for each with the given name
        // and status code.
        #[derive(poem_openapi::ApiResponse)]
        pub enum $enum_name<T: poem_openapi::types::ToJSON + Send + Sync> {
            $(
            #[oai(status = $status)]
            $name(
                // We use just regular u64 here instead of U64 since all header
                // values are implicitly strings anyway.
                $crate::response::CedraResponseContent<T>,
                /// Chain ID of the current chain
                #[oai(header = "X-Cedra-Chain-Id")] u8,
                /// Current ledger version of the chain
                #[oai(header = "X-Cedra-Ledger-Version")] u64,
                /// Oldest non-pruned ledger version of the chain
                #[oai(header = "X-Cedra-Ledger-Oldest-Version")] u64,
                /// Current timestamp of the chain
                #[oai(header = "X-Cedra-Ledger-TimestampUsec")] u64,
                /// Current epoch of the chain
                #[oai(header = "X-Cedra-Epoch")] u64,
                /// Current block height of the chain
                #[oai(header = "X-Cedra-Block-Height")] u64,
                /// Oldest non-pruned block height of the chain
                #[oai(header = "X-Cedra-Oldest-Block-Height")] u64,
                /// The cost of the call in terms of gas
                #[oai(header = "X-Cedra-Gas-Used")] Option<u64>,
                /// Cursor to be used for endpoints that support cursor-based
                /// pagination. Pass this to the `start` field of the endpoint
                /// on the next call to get the next page of results.
                #[oai(header = "X-Cedra-Cursor")] Option<String>,
            ),
            )*
        }

        // Generate an enum that captures all the different status codes that
        // this response type supports. To explain this funky syntax, if you
        // named the main enum MyResponse, this would become MyResponseCode.
        pub enum [<$enum_name Status>] {
            $(
            $name,
            )*
        }

        // Generate a From impl that builds a response from CedraResponseContent.
        // Each variant in the main enum takes in the same argument, so the macro
        // is really just helping us enumerate and build each variant. We use this
        // in the other From impls.
        impl <T: poem_openapi::types::ToJSON + Send + Sync> From<($crate::response::CedraResponseContent<T>, &cedra_api_types::LedgerInfo, [<$enum_name Status>])>
            for $enum_name<T>
        {
            fn from(
                (value, ledger_info, status): (
                    $crate::response::CedraResponseContent<T>,
                    &cedra_api_types::LedgerInfo,
                    [<$enum_name Status>]
                ),
            ) -> Self {
                match status {
                    $(
                    [<$enum_name Status>]::$name => {
                        $enum_name::$name(
                            value,
                            ledger_info.chain_id,
                            ledger_info.ledger_version.into(),
                            ledger_info.oldest_ledger_version.into(),
                            ledger_info.ledger_timestamp.into(),
                            ledger_info.epoch.into(),
                            ledger_info.block_height.into(),
                            ledger_info.oldest_block_height.into(),
                            None,
                            None,
                        )
                    },
                    )*
                }
            }
        }

        // Generate a From impl that builds a response from a Json<T> and friends.
        impl<T: poem_openapi::types::ToJSON + Send + Sync> From<(poem_openapi::payload::Json<T>, &cedra_api_types::LedgerInfo, [<$enum_name Status>])>
            for $enum_name<T>
        {
            fn from(
                (value, ledger_info, status): (poem_openapi::payload::Json<T>, &cedra_api_types::LedgerInfo, [<$enum_name Status>]),
            ) -> Self {
                let content = $crate::response::CedraResponseContent::Json(value);
                Self::from((content, ledger_info, status))
            }
        }

        // Generate a From impl that builds a response from a Bcs<Vec<u8>> and friends.
        impl<T: poem_openapi::types::ToJSON + Send + Sync> From<($crate::bcs_payload::Bcs, &cedra_api_types::LedgerInfo, [<$enum_name Status>])>
            for $enum_name<T>
        {
            fn from(
                (value, ledger_info, status): (
                    $crate::bcs_payload::Bcs,
                    &cedra_api_types::LedgerInfo,
                    [<$enum_name Status>]
                ),
            ) -> Self {
                let content = $crate::response::CedraResponseContent::Bcs(value);
                Self::from((content, ledger_info, status))
            }
        }

        // Generate a TryFrom impl that builds a response from a T, an AcceptType,
        // and all the other usual suspects. It expects to be called with a generic
        // parameter E: InternalError, with which we can build an internal error
        // response in case the BCS serialization fails.
        impl<T: poem_openapi::types::ToJSON + Send + Sync + serde::Serialize> $enum_name<T> {
            pub fn try_from_rust_value<E: $crate::response::InternalError>(
                (value, ledger_info, status, accept_type): (
                    T,
                    &cedra_api_types::LedgerInfo,
                    [<$enum_name Status>],
                    &$crate::accept_type::AcceptType
                ),
            ) -> Result<Self, E> {
                match accept_type {
                    AcceptType::Bcs => Ok(Self::from((
                        $crate::bcs_payload::Bcs(
                            bcs::to_bytes(&value)
                                .map_err(|e| E::internal_with_code(
                                    e,
                                    cedra_api_types::CedraErrorCode::InternalError,
                                    ledger_info
                                ))?
                        ),
                        ledger_info,
                        status
                    ))),
                    AcceptType::Json => Ok(Self::from((
                        poem_openapi::payload::Json(value),
                        ledger_info,
                        status
                    ))),
                }
            }

           pub fn try_from_json<E: $crate::response::InternalError>(
                (value, ledger_info, status): (
                    T,
                    &cedra_api_types::LedgerInfo,
                    [<$enum_name Status>],
                ),
            ) -> Result<Self, E> {
               Ok(Self::from((
                    poem_openapi::payload::Json(value),
                    ledger_info,
                    status
               )))
            }

            pub fn try_from_bcs<B: serde::Serialize, E: $crate::response::InternalError>(
                (value, ledger_info, status): (
                    B,
                    &cedra_api_types::LedgerInfo,
                    [<$enum_name Status>],
                ),
            ) -> Result<Self, E> {
               Ok(Self::from((
                    $crate::bcs_payload::Bcs(
                        bcs::to_bytes(&value)
                            .map_err(|e| E::internal_with_code(
                                e,
                                cedra_api_types::CedraErrorCode::InternalError,
                                ledger_info
                            ))?
                    ),
                    ledger_info,
                    status
               )))
            }

            pub fn try_from_encoded<E: $crate::response::InternalError>(
                (value, ledger_info, status): (
                    Vec<u8>,
                    &cedra_api_types::LedgerInfo,
                    [<$enum_name Status>],
                ),
            ) -> Result<Self, E> {
               Ok(Self::from((
                    $crate::bcs_payload::Bcs(
                        value
                    ),
                    ledger_info,
                    status
               )))
            }

            pub fn with_cursor(mut self, new_cursor: Option<cedra_types::state_store::state_key::StateKey>) -> Self {
                match self {
                    $(
                    [<$enum_name>]::$name(_, _, _, _, _, _, _, _, _, ref mut cursor) => {
                        *cursor = new_cursor.map(|c| cedra_api_types::StateKeyWrapper::from(c).to_string());
                    }
                    )*
                }
                self
            }

            pub fn with_gas_used(mut self, new_gas_used: Option<u64>) -> Self {
                match self {
                    $(
                    [<$enum_name>]::$name(_, _, _, _, _, _, _, _, ref mut gas_used, _) => {
                        *gas_used = new_gas_used;
                    }
                    )*
                }
                self
            }
        }
        }
    };
}

// Generate a success response that only has an option for 200.
generate_success_response!(BasicResponse, (200, Ok));

// Generate traits defining a "from" function for each of these status types.
// The error response then impls these traits for each status type they mention.
generate_error_traits!(
    BadRequest,
    Gone,
    NotFound,
    Forbidden,
    PayloadTooLarge,
    Internal,
    InsufficientStorage,
    ServiceUnavailable
);

// Group these common errors together
pub trait StdApiError: NotFoundError + GoneError + InternalError + ServiceUnavailableError {}
impl<T> StdApiError for T where
    T: NotFoundError + GoneError + InternalError + ServiceUnavailableError
{
}

// Generate an error response that only has options for 400 and 500.
generate_error_response!(
    BasicError,
    (400, BadRequest),
    (403, Forbidden),
    (500, Internal),
    (503, ServiceUnavailable)
);

// This type just simplifies using BasicResponse and BasicError together.
pub type BasicResult<T> = poem::Result<BasicResponse<T>, BasicError>;

// As above but with 404.
generate_error_response!(
    BasicErrorWith404,
    (400, BadRequest),
    (403, Forbidden),
    (404, NotFound),
    (410, Gone),
    (500, Internal),
    (503, ServiceUnavailable)
);
pub type BasicResultWith404<T> = poem::Result<BasicResponse<T>, BasicErrorWith404>;

// Just this one helper for a specific kind of 404.
pub fn build_not_found<S: Display, E: NotFoundError>(
    resource: &str,
    identifier: S,
    error_code: CedraErrorCode,
    ledger_info: &LedgerInfo,
) -> E {
    E::not_found_with_code(
        format!("{} not found by {}", resource, identifier),
        error_code,
        ledger_info,
    )
}

pub fn json_api_disabled<S: Display, E: ForbiddenError>(identifier: S) -> E {
    E::forbidden_with_code_no_info(
        format!(
            "{} with JSON output is disabled on this endpoint",
            identifier
        ),
        CedraErrorCode::ApiDisabled,
    )
}

pub fn bcs_api_disabled<S: Display, E: ForbiddenError>(identifier: S) -> E {
    E::forbidden_with_code_no_info(
        format!(
            "{} with BCS output is disabled on this endpoint",
            identifier
        ),
        CedraErrorCode::ApiDisabled,
    )
}

pub fn api_disabled<S: Display, E: ForbiddenError>(identifier: S) -> E {
    E::forbidden_with_code_no_info(
        format!("{} is disabled on this endpoint", identifier),
        CedraErrorCode::ApiDisabled,
    )
}

pub fn api_forbidden<S: Display, E: ForbiddenError>(identifier: S, extra_help: S) -> E {
    E::forbidden_with_code_no_info(
        format!("{} is not allowed. {}", identifier, extra_help),
        CedraErrorCode::ApiDisabled,
    )
}

pub fn version_not_found<E: NotFoundError>(ledger_version: u64, ledger_info: &LedgerInfo) -> E {
    build_not_found(
        "Ledger version",
        format!("Ledger version({})", ledger_version),
        CedraErrorCode::VersionNotFound,
        ledger_info,
    )
}

pub fn transaction_not_found_by_version<E: NotFoundError>(
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Transaction",
        format!("Ledger version({})", ledger_version),
        CedraErrorCode::TransactionNotFound,
        ledger_info,
    )
}

pub fn transaction_not_found_by_hash<E: NotFoundError>(
    hash: HashValue,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Transaction",
        format!("Transaction hash({})", hash),
        CedraErrorCode::TransactionNotFound,
        ledger_info,
    )
}

pub fn version_pruned<E: GoneError>(ledger_version: u64, ledger_info: &LedgerInfo) -> E {
    E::gone_with_code(
        format!("Ledger version({}) has been pruned", ledger_version),
        CedraErrorCode::VersionPruned,
        ledger_info,
    )
}

pub fn account_not_found<E: NotFoundError>(
    address: Address,
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Account",
        format!(
            "Address({}) and Ledger version({})",
            address, ledger_version
        ),
        CedraErrorCode::AccountNotFound,
        ledger_info,
    )
}

pub fn resource_not_found<E: NotFoundError>(
    address: Address,
    struct_tag: &StructTag,
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Resource",
        format!(
            "Address({}), Struct tag({}) and Ledger version({})",
            address, struct_tag, ledger_version
        ),
        CedraErrorCode::ResourceNotFound,
        ledger_info,
    )
}

pub fn module_not_found<E: NotFoundError>(
    address: Address,
    module_name: &IdentStr,
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Module",
        format!(
            "Address({}), Module name({}) and Ledger version({})",
            address, module_name, ledger_version
        ),
        CedraErrorCode::ModuleNotFound,
        ledger_info,
    )
}

pub fn struct_field_not_found<E: NotFoundError>(
    address: Address,
    struct_tag: &StructTag,
    field_name: &Identifier,
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Struct Field",
        format!(
            "Address({}), Struct tag({}), Field name({}) and Ledger version({})",
            address, struct_tag, field_name, ledger_version
        ),
        CedraErrorCode::StructFieldNotFound,
        ledger_info,
    )
}

pub fn table_item_not_found<E: NotFoundError>(
    table_handle: Address,
    table_key: &Value,
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Table Item",
        format!(
            "Table handle({}), Table key({}) and Ledger version({})",
            table_handle, table_key, ledger_version
        ),
        CedraErrorCode::TableItemNotFound,
        ledger_info,
    )
}

pub fn block_not_found_by_height<E: NotFoundError>(
    block_height: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Block",
        format!("Block height({})", block_height,),
        CedraErrorCode::BlockNotFound,
        ledger_info,
    )
}

pub fn block_not_found_by_version<E: NotFoundError>(
    ledger_version: u64,
    ledger_info: &LedgerInfo,
) -> E {
    build_not_found(
        "Block",
        format!("Ledger version({})", ledger_version,),
        CedraErrorCode::BlockNotFound,
        ledger_info,
    )
}

pub fn block_pruned_by_height<E: GoneError>(block_height: u64, ledger_info: &LedgerInfo) -> E {
    E::gone_with_code(
        format!("Block({}) has been pruned", block_height),
        CedraErrorCode::BlockPruned,
        ledger_info,
    )
}
