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
    let signature = did.sign_message(message);
    assert!(did.verify_signature(message, &signature));
}
