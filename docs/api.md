# ICN API Reference

## Transaction Types

### Resource Transactions
```rust
CoopResource {
    resource_id: String,
    operation: ResourceOperation,
    metadata: ResourceMetadata,
    did_proof: DidProof,
}
```

### Governance Transactions
```rust
GovernanceTransaction::SubmitProposal(proposal)
GovernanceTransaction::CastVote(vote)
GovernanceTransaction::FinalizeProposal(id)
```

## Storage Interface

### On-Chain Storage
Limited to small metadata and proofs.

### Off-Chain Storage
Uses IPFS/Filecoin with on-chain references:
```rust
StorageReference {
    location: StorageType,
    metadata: ResourceMetadata,
    access_control: AccessControl,
}
```

## Authentication

All transactions require DID-based authentication:
```rust
DidProof {
    did: String,
    signature: String,
    timestamp: u64,
}
```
