use secp256k1::SecretKey;
use rand::thread_rng;
use crate::Algorithm;
use crate::identity::did::creation::DID;
use crate::identity::did::serialization::SerializableDID;
use crate::identity::did::validation::DID as ValidationDID;

#[test]
fn test_did_creation() {
    let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    assert_eq!(did.id, "did:example:123");
}

#[test]
fn test_did_serialization() {
    let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}

#[test]
fn test_did_sign_and_verify() {
    let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify signature"));
}

#[test]
fn test_did_creation_rsa() {
    let did = DID::new("did:example:456".to_string(), Algorithm::RSA);
    assert_eq!(did.id, "did:example:456");
}

#[test]
fn test_did_serialization_rsa() {
    let did = DID::new("did:example:456".to_string(), Algorithm::RSA);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}

#[test]
fn test_did_sign_and_verify_rsa() {
    let did = DID::new("did:example:456".to_string(), Algorithm::RSA);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify signature"));
}

#[test]
fn test_did_creation_ecdsa() {
    let did = DID::new("did:example:789".to_string(), Algorithm::ECDSA);
    assert_eq!(did.id, "did:example:789");
}

#[test]
fn test_did_serialization_ecdsa() {
    let did = DID::new("did:example:789".to_string(), Algorithm::ECDSA);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}

#[test]
fn test_did_sign_and_verify_ecdsa() {
    let did = DID::new("did:example:789".to_string(), Algorithm::ECDSA);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify signature"));
}

#[test]
fn test_did_creation_kyber() {
    let did = DID::new("did:example:101".to_string(), Algorithm::Kyber);
    assert_eq!(did.id, "did:example:101");
}

#[test]
fn test_did_serialization_kyber() {
    let did = DID::new("did:example:101".to_string(), Algorithm::Kyber);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}

#[test]
fn test_did_sign_and_verify_kyber() {
    let did = DID::new("did:example:101".to_string(), Algorithm::Kyber);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify signature"));
}

#[test]
fn test_did_creation_dilithium() {
    let did = DID::new("did:example:202".to_string(), Algorithm::Dilithium);
    assert_eq!(did.id, "did:example:202");
}

#[test]
fn test_did_serialization_dilithium() {
    let did = DID::new("did:example:202".to_string(), Algorithm::Dilithium);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}

#[test]
fn test_did_sign_and_verify_dilithium() {
    let did = DID::new("did:example:202".to_string(), Algorithm::Dilithium);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify signature"));
}

#[test]
fn test_did_creation_falcon() {
    let did = DID::new("did:example:303".to_string(), Algorithm::Falcon);
    assert_eq!(did.id, "did:example:303");
}

#[test]
fn test_did_serialization_falcon() {
    let did = DID::new("did:example:303".to_string(), Algorithm::Falcon);
    let serializable_did: SerializableDID = (&did).into();
    let deserialized_did: DID = (&serializable_did).into();
    assert_eq!(did.id, deserialized_did.id);
}

#[test]
fn test_did_sign_and_verify_falcon() {
    let did = DID::new("did:example:303".to_string(), Algorithm::Falcon);
    let message = b"test message";
    let signature = did.sign_message(message).expect("Failed to sign message");
    assert!(did.verify_signature(message, &signature).expect("Failed to verify signature"));
}

#[test]
fn test_key_rotation() {
    let mut did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
    let old_public_key = did.public_key.clone();
    did.rotate_key().expect("Failed to rotate key");
    assert_ne!(old_public_key, did.public_key);
}
