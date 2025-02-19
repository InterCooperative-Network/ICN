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

#### Example
```rust
let resource_transaction = CoopResource {
    resource_id: "resource123".to_string(),
    operation: ResourceOperation::Allocate,
    metadata: ResourceMetadata {
        description: "Allocating resource for project X".to_string(),
        quantity: 100,
    },
    did_proof: DidProof {
        did: "did:icn:example".to_string(),
        signature: "signature123".to_string(),
        timestamp: 1637356800,
    },
};
```

### Governance Transactions
```rust
GovernanceTransaction::SubmitProposal(proposal)
GovernanceTransaction::CastVote(vote)
GovernanceTransaction::FinalizeProposal(id)
```

#### Example
```rust
let proposal = Proposal {
    id: "proposal123".to_string(),
    title: "New Governance Policy".to_string(),
    description: "Proposal to implement a new governance policy".to_string(),
    created_by: "did:icn:example".to_string(),
    ends_at: 1637356800,
};

let governance_transaction = GovernanceTransaction::SubmitProposal(proposal);
```

## Storage Interface

### On-Chain Storage
Limited to small metadata and proofs.

#### Example
```rust
let on_chain_storage = OnChainStorage {
    metadata: "Small metadata".to_string(),
    proof: "proof123".to_string(),
};
```

### Off-Chain Storage
Uses IPFS/Filecoin with on-chain references:
```rust
StorageReference {
    location: StorageType,
    metadata: ResourceMetadata,
    access_control: AccessControl,
}
```

#### Example
```rust
let off_chain_storage = StorageReference {
    location: StorageType::IPFS("ipfs://example".to_string()),
    metadata: ResourceMetadata {
        description: "Large file stored off-chain".to_string(),
        quantity: 1,
    },
    access_control: AccessControl {
        owner_did: "did:icn:example".to_string(),
        allowed_coops: vec!["coop1".to_string(), "coop2".to_string()],
        permissions: Permissions::Read,
    },
};
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

### DID-Based Authentication Process
1. **Generate DID**: Create a Decentralized Identifier (DID) for the user or entity.
2. **Sign Transaction**: Use the private key associated with the DID to sign the transaction.
3. **Verify Signature**: The system verifies the signature using the public key associated with the DID.
4. **Timestamp Validation**: Ensure the timestamp is within an acceptable range to prevent replay attacks.

#### Example
```rust
let did_proof = DidProof {
    did: "did:icn:example".to_string(),
    signature: "signature123".to_string(),
    timestamp: 1637356800,
};
```

## Error Handling

### Common Errors
- **InvalidDID**: The provided DID is not valid or not registered.
- **InsufficientReputation**: The user does not have enough reputation to perform the action.
- **SignatureVerificationFailed**: The signature verification process failed.
- **ResourceNotFound**: The requested resource does not exist.
- **UnauthorizedAccess**: The user does not have the necessary permissions to access the resource.

### Error Resolution
- **InvalidDID**: Ensure the DID is correctly formatted and registered in the system.
- **InsufficientReputation**: Increase the user's reputation by participating in cooperative activities.
- **SignatureVerificationFailed**: Verify that the correct private key was used to sign the transaction.
- **ResourceNotFound**: Check the resource ID and ensure it exists in the system.
- **UnauthorizedAccess**: Request the necessary permissions from the resource owner or cooperative.

## Glossary of Terms

- **DID (Decentralized Identifier)**: A unique identifier used to represent an entity within the ICN.
- **Proof of Cooperation (PoC)**: A consensus mechanism used to validate transactions in a cooperative model.
- **Resource Allocation**: The process of distributing resources among cooperatives.
- **Governance Transaction**: A transaction related to the governance of the cooperative, such as submitting proposals or casting votes.
- **On-Chain Storage**: Storage of small metadata and proofs directly on the blockchain.
- **Off-Chain Storage**: Storage of large files using external storage solutions like IPFS/Filecoin, with references stored on the blockchain.
- **Reputation**: A score representing the trustworthiness and contributions of a user within the cooperative.
- **Access Control**: Mechanisms to restrict access to resources based on permissions and roles.
- **Signature Verification**: The process of verifying the authenticity of a transaction using cryptographic signatures.
