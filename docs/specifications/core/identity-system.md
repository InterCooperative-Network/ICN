---
authors:
  - Matt Faherty
date: '2024-11-18'
status: draft
title: Identity System Specification
type: specification
version: 1.2.0
---

# Identity System Documentation

## Overview
The Identity System is a core component of the Inter-Cooperative Network (ICN). It provides decentralized identity management through the use of Decentralized Identifiers (DIDs). This system facilitates secure interactions and permissions management across the network, allowing for authentication and authorization of cooperative members.

### Purpose
- **Decentralized Identity Management**: The Identity System provides a framework for creating and managing DIDs for all participants in the network.
- **Authentication and Access Control**: It supports secure authentication processes and manages permissions associated with each DID.
- **Integration with Cooperative Services**: The system ties into other ICN services like resource sharing, governance, and consensus.

## 1. System Components

### 1.1 Decentralized Identifiers (DIDs)
A DID is a cryptographic identifier representing users or entities within the ICN. DIDs are generated using cryptographic primitives for uniqueness and security.

#### DID Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String,
    #[serde(serialize_with = "serialize_public_key")]
    #[serde(deserialize_with = "deserialize_public_key")]
    pub public_key: PublicKey,
}
```
- **id**: A unique identifier for each DID.
- **public_key**: The public key used for identity verification.

### 1.2 Identity System
The Identity System tracks registered DIDs, manages their permissions, and supports verification processes.

#### Identity System Structure
```rust
#[derive(Clone)]
pub struct IdentitySystem {
    permissions: HashMap<String, Vec<String>>,
    registered_dids: HashMap<String, DID>,
    reputation_scores: HashMap<String, i64>,
    last_activity: HashMap<String, SystemTime>,
}
```
- **permissions**: Maps each DID to its assigned permissions.
- **registered_dids**: Stores the registered DIDs for verification and reference.
- **reputation_scores**: Tracks the reputation scores of each DID.
- **last_activity**: Records the last activity time of each DID.

## 2. Key Methods

### 2.1 DID Generation
DIDs are generated using cryptographic methods to ensure both uniqueness and security. The function `generate_random` is used to create a new DID with an associated secret key.

```rust
pub fn generate_random(id: String) -> (DID, SecretKey) {
    let secp = Secp256k1::new();
    let mut rng = thread_rng();
    let mut secret_key_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_key_bytes);
    let secret_key = SecretKey::from_slice(&secret_key_bytes).expect("Valid key");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    (DID { id, public_key }, secret_key)
}
```
- **Input**: `id` - A string identifier for the DID.
- **Output**: A tuple containing the generated `DID` and its associated `SecretKey`.

### 2.2 Register DID
The `register_did` method registers a DID with the Identity System and assigns it permissions.

```rust
pub fn register_did(&mut self, did: DID, permissions: Vec<String>, initial_reputation: i64) {
    self.permissions.insert(did.id.clone(), permissions);
    self.registered_dids.insert(did.id.clone(), did);
    self.reputation_scores.insert(did.id.clone(), initial_reputation);
    self.last_activity.insert(did.id.clone(), SystemTime::now());
}
```
- **Input**: `did` - The DID to be registered, `permissions` - A list of permissions to assign, `initial_reputation` - The initial reputation score.
- **Functionality**: Adds the DID and permissions to the Identity System for future reference.

### 2.3 Get Permissions
The `get_permissions` method retrieves the permissions associated with a particular DID.

```rust
pub fn get_permissions(&self, did: &str) -> Vec<String> {
    self.permissions.get(did).cloned().unwrap_or_default()
}
```
- **Input**: `did` - The identifier for the DID.
- **Output**: A list of strings representing the permissions associated with the DID.

### 2.4 Verify DID Registration
The `is_registered` method checks if a DID is registered with the Identity System.

```rust
pub fn is_registered(&self, did: &str) -> bool {
    self.registered_dids.contains_key(did)
}
```
- **Input**: `did` - The DID to verify.
- **Output**: A boolean indicating whether the DID is registered.

### 2.5 Authentication
The Identity System authenticates DIDs using cryptographic verification. The `verify_did` function checks if the DID’s public key matches the stored public key.

```rust
pub fn verify_did(&self, did: &str, message: &[u8], signature: &secp256k1::Signature) -> bool {
    if let Some(did_obj) = self.registered_dids.get(did) {
        let secp = secp256k1::Secp256k1::new();
        let msg = secp256k1::Message::from_slice(message).expect("32 bytes");
        secp.verify(&msg, signature, &did_obj.public_key).is_ok()
    } else {
        false
    }
}
```
- **Input**: `did` - The DID to verify, `message` - The message to verify, `signature` - The signature to verify.
- **Output**: A boolean indicating whether the DID is valid.

## 3. Permission Management

### 3.1 Adding and Removing Permissions
The Identity System allows dynamic updating of permissions for registered DIDs.

#### Add Permission
```rust
pub fn add_permission(&mut self, did: &str, permission: String) {
    if let Some(perms) = self.permissions.get_mut(did) {
        if !perms.contains(&permission) {
            perms.push(permission);
        }
    }
}
```
- **Input**: `did` - The DID to update, `permission` - The permission to add.

#### Remove Permission
```rust
pub fn remove_permission(&mut self, did: &str, permission: &str) {
    if let Some(perms) = self.permissions.get_mut(did) {
        perms.retain(|p| p != permission);
    }
}
```
- **Input**: `did` - The DID to update, `permission` - The permission to remove.

## 4. Security Considerations

### 4.1 Reputation-Based Access
Reputation scores are integrated into the Identity System to enhance security and ensure responsible access. Users must maintain a positive reputation to retain or gain permissions for critical operations.

### 4.2 Cryptographic Security
The Identity System uses Secp256k1 for generating DIDs and verifying public keys. Future updates may integrate quantum-resistant cryptographic methods to ensure long-term security.

### 4.3 Graceful Error Handling
The Identity System handles errors gracefully, providing meaningful error messages that aid in debugging while maintaining security.

## 5. Implementation Guidelines

### 5.1 Performance Requirements
- **Efficient Data Retrieval**: Use hash maps for O(1) lookup times for DID verification and permission retrieval.
- **Scalability**: Ensure the system is optimized for handling thousands of DIDs without significant performance overhead.

### 5.2 Testing Requirements
- **Unit Tests**: Cover all core methods such as `register_did`, `add_permission`, and `verify_did`.
- **Integration Tests**: Verify that the Identity System integrates smoothly with other components like governance and reputation modules.

## 6. Future Considerations

### 6.1 Quantum-Resistant Keys
To future-proof the system against advancements in quantum computing, a migration to quantum-resistant algorithms (e.g., CRYSTALS-Dilithium) is under consideration.

### 6.2 DID Lifecycle Management
Adding lifecycle management for DIDs, including deactivation, expiration, and renewal, will enhance security and prevent unauthorized use of stale identities.

## 7. Dynamic Recalibration and Reputation Decay

### 7.1 Dynamic Recalibration
To ensure dynamic recalibration of reputation scores, the following approaches are considered:

- **Continuous Monitoring**: A system is implemented that continuously monitors the activities and contributions of participants. This can be achieved by integrating the reputation system with various components of the network, such as the consensus mechanism, governance, and resource sharing.
- **Periodic Updates**: Periodic updates are scheduled to recalculate reputation scores based on recent activities and contributions. This can be done using a background task or a scheduled job that runs at regular intervals.
- **Event-Driven Recalibration**: An event-driven system is implemented that recalibrates reputation scores in response to specific events, such as successful block proposals, voting participation, or resource sharing.

### 7.2 Reputation Decay
A decay mechanism is introduced that gradually reduces reputation scores over time if participants do not engage in positive activities. This encourages continuous participation and prevents reputation scores from remaining static.

- **Decay Function**: The decay rate is applied periodically (e.g., monthly) to reduce scores by a small percentage if no positive actions are recorded.
- **Decay Rate Configuration**: The decay rate can be configured to adapt to different community dynamics and participation levels.
- **Decay Exemptions**: Certain participants or activities can be exempted from decay to ensure critical contributors are not unfairly penalized for temporary inactivity.

## 8. Testing and Validation Procedures

### 8.1 DID Generation and Validation
Comprehensive tests are implemented to ensure the correct generation and validation of DIDs. These tests cover various scenarios, including valid and invalid DIDs, to ensure the system's robustness.

#### DID Creation Test
```rust
#[test]
fn test_did_creation() {
    let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    assert_eq!(did.id, "did:example:123");
}
```
- **Purpose**: Verify that a DID is created correctly with the specified ID.

#### DID Serialization Test
```rust
#[test]
fn test_did_serialization() {
    let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}
```
- **Purpose**: Ensure that a DID can be serialized and deserialized correctly.

#### DID Sign and Verify Test
```rust
#[test]
fn test_did_sign_and_verify() {
    let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    let message = b"test message";
    let signature = did.sign_message(message);
    assert!(did.verify_signature(message, &signature));
}
```
- **Purpose**: Validate that a DID can sign a message and verify the signature correctly.

### 8.2 Integration Tests
Integration tests are written to confirm that the DID validation works correctly within the blockchain system. These tests include scenarios for valid and invalid DIDs, ensuring the system's robustness.

#### DID Validation Integration Test
```rust
#[tokio::test]
async fn test_did_validation() {
    let did = DID::new("did:icn:test".to_string(), &SecretKey::new(&mut thread_rng()));
    let message = b"test message";
    let signature = did.sign_message(message);
    assert!(did.verify_signature(message, &signature));
}
```
- **Purpose**: Verify that the DID validation works correctly within the blockchain system.

## Appendix

### A. Summary of Methods
- **Generate DID**: Creates a new decentralized identifier.
- **Register DID**: Adds a DID to the registry with permissions.
- **Verify DID**: Confirms a DID’s validity via cryptographic checks.
- **Permission Management**: Add or remove permissions for a DID as roles change.
- **Dynamic Recalibration**: Continuously updates reputation scores based on ongoing activities and contributions.
- **Reputation Decay**: Gradually reduces reputation scores over time if participants do not engage in positive activities.

### B. Modular Structure

The identity management modules are now split into smaller submodules for better separation of concerns. Below is the updated structure:

#### identity/did.rs
- **creation**: Handles the creation of DIDs.
- **serialization**: Manages the serialization and deserialization of DIDs.
- **validation**: Provides methods for signing and verifying messages.

#### identity/identity_system.rs
- **permission_handling**: Manages permissions associated with DIDs.
- **role_management**: Handles role assignments and retrievals.
- **identity_verification**: Provides methods for verifying DIDs using cryptographic signatures.
- **reputation_management**: Manages reputation scores, dynamic recalibration, and reputation decay.
