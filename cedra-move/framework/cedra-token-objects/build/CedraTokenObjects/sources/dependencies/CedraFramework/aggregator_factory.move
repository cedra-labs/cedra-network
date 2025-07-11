/// This module provides foundations to create aggregators. Currently only
/// Cedra Framework (0x1) can create them, so this module helps to wrap
/// the constructor of `Aggregator` struct so that only a system account
/// can initialize one. In the future, this might change and aggregators
/// can be enabled for the public.
module cedra_framework::aggregator_factory {
    use std::error;

    use cedra_framework::system_addresses;
    use cedra_framework::aggregator::Aggregator;
    use cedra_std::table::{Self, Table};

    friend cedra_framework::genesis;
    friend cedra_framework::optional_aggregator;

    /// Aggregator factory is not published yet.
    const EAGGREGATOR_FACTORY_NOT_FOUND: u64 = 1;

    /// Aggregator V1 only supports limit == MAX_U128.
    const EAGG_V1_LIMIT_DEPRECATED: u64 = 2;

    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    /// Creates new aggregators. Used to control the numbers of aggregators in the
    /// system and who can create them. At the moment, only Cedra Framework (0x1)
    /// account can.
    struct AggregatorFactory has key {
        phantom_table: Table<address, u128>,
    }

    /// Creates a new factory for aggregators. Can only be called during genesis.
    public(friend) fun initialize_aggregator_factory(cedra_framework: &signer) {
        system_addresses::assert_cedra_framework(cedra_framework);
        let aggregator_factory = AggregatorFactory {
            phantom_table: table::new()
        };
        move_to(cedra_framework, aggregator_factory);
    }

    /// Creates a new aggregator instance which overflows on exceeding a `limit`.
    public(friend) fun create_aggregator_internal(): Aggregator acquires AggregatorFactory {
        assert!(
            exists<AggregatorFactory>(@cedra_framework),
            error::not_found(EAGGREGATOR_FACTORY_NOT_FOUND)
        );

        let aggregator_factory = borrow_global_mut<AggregatorFactory>(@cedra_framework);
        new_aggregator(aggregator_factory, MAX_U128)
    }

    #[deprecated]
    /// This is currently a function closed for public. This can be updated in the future by on-chain governance
    /// to allow any signer to call.
    public fun create_aggregator(account: &signer, limit: u128): Aggregator acquires AggregatorFactory {
        // deprecated. Currently used only in cedra-move/e2e-move-tests/src/tests/aggregator.data/pack/sources/aggregator_test.move

        // Only Cedra Framework (0x1) account can call this for now.
        system_addresses::assert_cedra_framework(account);
        assert!(
            limit == MAX_U128,
            error::invalid_argument(EAGG_V1_LIMIT_DEPRECATED)
        );
        create_aggregator_internal()
    }

    /// Returns a new aggregator.
    native fun new_aggregator(aggregator_factory: &mut AggregatorFactory, limit: u128): Aggregator;

    #[test_only]
    public fun create_aggregator_for_test(): Aggregator acquires AggregatorFactory {
        create_aggregator_internal()
    }

    #[test_only]
    public fun initialize_aggregator_factory_for_test(cedra_framework: &signer) {
        initialize_aggregator_factory(cedra_framework);
    }

    #[test_only]
    public fun aggregator_factory_exists_for_testing(): bool {
        exists<AggregatorFactory>(@cedra_framework)
    }
}
