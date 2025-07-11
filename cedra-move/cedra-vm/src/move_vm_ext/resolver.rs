// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

use cedra_aggregator::resolver::{AggregatorV1Resolver, DelayedFieldResolver};
use cedra_table_natives::TableResolver;
use cedra_types::{on_chain_config::ConfigStorage, state_store::state_key::StateKey};
use cedra_vm_types::resolver::{
    ExecutorView, ResourceGroupSize, ResourceGroupView, StateStorageView,
};
use bytes::Bytes;
use move_binary_format::errors::PartialVMResult;
use move_core_types::language_storage::StructTag;
use move_vm_types::resolver::ResourceResolver;
use std::collections::{BTreeMap, HashMap};

/// A general resolver used by CedraVM. Allows to implement custom hooks on
/// top of storage, e.g. get resources from resource groups, etc.
/// MoveResolver implements ResourceResolver and ModuleResolver
pub trait CedraMoveResolver:
    AggregatorV1Resolver
    + ConfigStorage
    + DelayedFieldResolver
    + ResourceResolver
    + ResourceGroupResolver
    + StateStorageView<Key = StateKey>
    + TableResolver
    + AsExecutorView
    + AsResourceGroupView
{
}

pub trait ResourceGroupResolver {
    fn release_resource_group_cache(&self)
        -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>>;

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize>;

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize>;

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool>;
}

pub trait AsExecutorView {
    fn as_executor_view(&self) -> &dyn ExecutorView;
}

pub trait AsResourceGroupView {
    fn as_resource_group_view(&self) -> &dyn ResourceGroupView;
}
