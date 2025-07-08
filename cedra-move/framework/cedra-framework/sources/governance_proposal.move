/// Define the GovernanceProposal that will be used as part of on-chain governance by CedraGovernance.
///
/// This is separate from the CedraGovernance module to avoid circular dependency between CedraGovernance and Stake.
module cedra_framework::governance_proposal {
    friend cedra_framework::cedra_governance;

    struct GovernanceProposal has store, drop {}

    /// Create and return a GovernanceProposal resource. Can only be called by CedraGovernance
    public(friend) fun create_proposal(): GovernanceProposal {
        GovernanceProposal {}
    }

    /// Useful for CedraGovernance to create an empty proposal as proof.
    public(friend) fun create_empty_proposal(): GovernanceProposal {
        create_proposal()
    }

    #[test_only]
    public fun create_test_proposal(): GovernanceProposal {
        create_empty_proposal()
    }
}
