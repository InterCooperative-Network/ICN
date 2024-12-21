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
