spec cedra_framework::aggregator_factory {
    use cedra_framework::aggregator;
    /// <high-level-req>
    /// No.: 1
    /// Requirement: During the module's initialization, it guarantees that the Cedra framework is the caller and that the
    /// AggregatorFactory resource will move under the Cedra framework account.
    /// Criticality: High
    /// Implementation: The initialize function is responsible for establishing the initial state of the module by
    /// creating the AggregatorFactory resource, indicating its presence within the module's context. Subsequently, the
    /// resource transfers to the Cedra framework account.
    /// Enforcement: Formally verified via [high-level-req-1](initialize_aggregator_factory).
    ///
    /// No.: 2
    /// Requirement: To create a new aggregator instance, the aggregator factory must already be initialized and exist
    /// under the Cedra account.
    /// Criticality: High
    /// Implementation: The create_aggregator_internal function asserts that AggregatorFactory exists for the Cedra
    /// account.
    /// Enforcement: Formally verified via [high-level-req-2](CreateAggregatorInternalAbortsIf).
    ///
    /// No.: 3
    /// Requirement: Only the Cedra framework address may create an aggregator instance currently.
    /// Criticality: Low
    /// Implementation: The create_aggregator function ensures that the address calling it is the Cedra framework
    /// address.
    /// Enforcement: Formally verified via [high-level-req-3](create_aggregator).
    ///
    /// No.: 4
    /// Requirement: The creation of new aggregators should be done correctly.
    /// Criticality: High
    /// Implementation: The native new_aggregator function correctly creates a new aggregator.
    /// Enforcement: The new_aggregator native function has been manually audited.
    /// </high-level-req>
    ///
    spec module {
        pragma aborts_if_is_strict;
    }

    spec new_aggregator(aggregator_factory: &mut AggregatorFactory, limit: u128): Aggregator {
        pragma opaque;
        aborts_if false;
        ensures result == spec_new_aggregator(limit);
        ensures aggregator::spec_get_limit(result) == limit;
    }

    /// Make sure the caller is @cedra_framework.
    /// AggregatorFactory is not under the caller before creating the resource.
    spec initialize_aggregator_factory(cedra_framework: &signer) {
        use std::signer;
        let addr = signer::address_of(cedra_framework);
        aborts_if addr != @cedra_framework;
        aborts_if exists<AggregatorFactory>(addr);
        /// [high-level-req-1]
        ensures exists<AggregatorFactory>(addr);
    }

    spec create_aggregator_internal(): Aggregator {
        /// [high-level-req-2]
        include CreateAggregatorInternalAbortsIf;
        ensures aggregator::spec_get_limit(result) == MAX_U128;
        ensures aggregator::spec_aggregator_get_val(result) == 0;
    }
    spec schema CreateAggregatorInternalAbortsIf {
        aborts_if !exists<AggregatorFactory>(@cedra_framework);
    }

    /// Make sure the caller is @cedra_framework.
    /// AggregatorFactory existed under the @cedra_framework when Creating a new aggregator.
    spec create_aggregator(account: &signer, limit: u128): Aggregator {
        use std::signer;
        let addr = signer::address_of(account);
        /// [high-level-req-3]
        aborts_if addr != @cedra_framework;
        aborts_if limit != MAX_U128;
        aborts_if !exists<AggregatorFactory>(@cedra_framework);
    }

    spec native fun spec_new_aggregator(limit: u128): Aggregator;

}
