spec diem_framework::aggregator_factory {
    use diem_framework::aggregator;
    spec module {
        pragma aborts_if_is_strict;
    }

    spec new_aggregator(aggregator_factory: &mut AggregatorFactory, limit: u128): Aggregator {
        pragma opaque;
        aborts_if false;
        ensures result == spec_new_aggregator(limit);
        ensures aggregator::spec_get_limit(result) == limit;
    }

    /// Make sure the caller is @diem_framework.
    /// AggregatorFactory is not under the caller before creating the resource.
    spec initialize_aggregator_factory(diem_framework: &signer) {
        use std::signer;
        let addr = signer::address_of(diem_framework);
        aborts_if addr != @diem_framework;
        aborts_if exists<AggregatorFactory>(addr);
        ensures exists<AggregatorFactory>(addr);
    }

    spec create_aggregator_internal(limit: u128): Aggregator {
        include CreateAggregatorInternalAbortsIf;
        ensures aggregator::spec_get_limit(result) == limit;
        ensures aggregator::spec_aggregator_get_val(result) == 0;
    }
    spec schema CreateAggregatorInternalAbortsIf {
        aborts_if !exists<AggregatorFactory>(@diem_framework);
    }

    /// Make sure the caller is @diem_framework.
    /// AggregatorFactory existed under the @diem_framework when Creating a new aggregator.
    spec create_aggregator(account: &signer, limit: u128): Aggregator {
        use std::signer;
        let addr = signer::address_of(account);
        aborts_if addr != @diem_framework;
        aborts_if !exists<AggregatorFactory>(@diem_framework);
    }

    spec native fun spec_new_aggregator(limit: u128): Aggregator;

}
