---
authors:
  - Matt Faherty
date: '2024-11-18'
status: draft
title: Identity System
version: 1.1.0
---

# Identity System Documentation

## 1. Overview

The Identity System within ICN is a core infrastructure component that enables secure, decentralized identity management for cooperatives, communities, and their members. It provides a framework for generating, managing, and authenticating Decentralized Identifiers (DIDs). This system is crucial for ensuring both identity integrity and secure, permissioned participation in ICN activities.

### 1.1 Purpose
- **Decentralized Authentication**: Provides secure, decentralized identity verification.
- **Role and Permission Management**: Associates roles and permissions with DIDs for cooperative governance and activities.
- **Accountability**: Ensures that participants can be uniquely identified while protecting their privacy.

## 2. Core Components

### 2.1 Data Structures

#### 2.1.1 DID Structure
The `DID` struct is the foundational element for ICN identities:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String,
    #[serde(serialize_with = "serialize_public_key")]
    #[serde(deserialize_with = "deserialize_public_key")]
    pub public_key: PublicKey,
}
```
- **id**: A unique string identifier for the DID.
- **public_key**: A public key associated with the DID, serialized for communication purposes.

### 2.2 Identity System Structure
The `IdentitySystem` struct handles the management and registration of DIDs, as well as role-based permissions.

```rust
pub struct IdentitySystem {
    permissions: HashMap<String, Vec<String>>,
    registered_dids: HashMap<String, DID>,
}
```
- **permissions**: Maps each DID to a list of permissions granted within the cooperative or community context.
- **registered_dids**: Stores all registered DIDs, ensuring each DID is uniquely identifiable.

## 3. Interfaces

### 3.1 DID Creation and Management

#### 3.1.1 DID Generation
- **Method**: `DID::generate_random(id: String) -> (Self, SecretKey)`
- **Purpose**: Generates a new DID using a randomly generated secret key.
  - **Input**: A unique identifier string.
  - **Output**: A DID instance and its associated `SecretKey`.
- **Process**:
  1. Generate a random 32-byte secret key.
  2. Create a public key from the secret key using the `secp256k1` library.
  3. Return the new DID and secret key pair.

```rust
pub fn generate_random(id: String) -> (Self, SecretKey) {
    let secp = Secp256k1::new();
    let mut rng = thread_rng();
    let mut secret_key_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_key_bytes);
    let secret_key = SecretKey::from_slice(&secret_key_bytes).expect("Random bytes should produce valid key");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    DID { id, public_key }, secret_key
}
```

#### 3.1.2 DID Registration
- **Method**: `IdentitySystem::register_did(did: DID, permissions: Vec<String>)`
- **Purpose**: Registers a new DID with specified permissions.
  - **Input**: A `DID` instance and a list of permissions.
  - **Process**: Adds the DID and associated permissions to `registered_dids` and `permissions` respectively.

```rust
pub fn register_did(&mut self, did: DID, permissions: Vec<String>) {
    self.permissions.insert(did.id.clone(), permissions);
    self.registered_dids.insert(did.id.clone(), did);
}
```

## 4. Behaviors

### 4.1 Role and Permission Management

#### 4.1.1 Permission Management
- **Method**: `IdentitySystem::add_permission(did: &str, permission: String)`
- **Purpose**: Grants an additional permission to a registered DID.
  - **Input**: The DID identifier and the permission to be added.
  - **Process**: Checks if the DID exists, then adds the permission if it is not already present.

#### 4.1.2 DID Authentication
- **Purpose**: The `Authentication` struct (currently minimal in code) is intended for developing methods that authenticate users based on their DIDs and verify their public-private key pairs.

## 5. Security Model

### 5.1 Public Key Management
The Identity System uses `secp256k1` elliptic curve cryptography to generate public and secret key pairs. The **DID** is associated with a `PublicKey`, while the `SecretKey` remains with the user for signing purposes.

### 5.2 Permission Validation
- **Enforcement**: Permissions are enforced by querying the `permissions` map during identity-based actions.
- **Scoped Access**: Only DIDs with the appropriate permissions may execute restricted operations, ensuring secure and compliant interactions within the cooperative.

## 6. Implementation Guidelines

### 6.1 Extensibility
- **Future Integration**: Expand the `IdentitySystem` to include DID document management and cryptographic proof capabilities for better interoperability.
- **Inter-Cooperative Identity Validation**: Introduce cross-cooperative identity validation for federated environments.

### 6.2 Security Requirements
- **Key Management**: Users must securely store their `SecretKey` to prevent unauthorized access.
- **Reputation and Trust**: DIDs should be linked to reputation scores that affect the level of trust and permissions granted.

## 7. Testing Requirements

- **Unit Tests**: Ensure correctness in DID generation, registration, and permission management.
- **Integration Tests**: Verify secure DID creation and permission checks across cooperative modules.
- **Security Tests**: Test scenarios where unauthorized actions are attempted, ensuring proper enforcement of permissions.

## Appendix

### A. Sample Usage

```rust
let mut identity_system = IdentitySystem::new();
let (new_did, secret_key) = DID::generate_random("did:icn:user1".to_string());
identity_system.register_did(new_did.clone(), vec!["vote", "create_proposal"]);
assert!(identity_system.is_registered(&new_did.id));
```

### B. Serialization and Deserialization
- **Serialization**: Public keys are serialized to strings for storage and transmission.
- **Deserialization**: The `deserialize_public_key` function recreates the `PublicKey` from a string representation.

