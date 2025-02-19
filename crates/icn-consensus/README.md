# ICN Consensus Engine

Implements the Proof of Cooperation consensus mechanism with democratic validator governance.

## Validator Selection

Validators are selected through a democratic process:
- Each cooperative can propose validators
- Proposals require majority approval
- Maximum validators per cooperative is enforced
- Minimum stake requirements apply

## Governance Rules

```rust
ValidatorRules {
    max_validators_per_coop: 2,
    min_stake_requirement: 100_000,
    election_period_blocks: 40320, // ~7 days
}
```

## Usage Example

```rust
let rules = ValidatorRules::default();
let engine = GovernanceEngine::new(rules);

// Submit validator proposal
let proposal = VoteProposal::new(
    ProposalType::AddValidator(validator_info)
);
engine.submit_proposal(proposal)?;
```
