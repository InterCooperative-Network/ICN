use secp256k1::{SecretKey, PublicKey, Secp256k1};
use rsa::{RSAPrivateKey, RSAPublicKey};
use ecdsa::{SigningKey, VerifyingKey};
use rand::thread_rng;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use thiserror::Error;
use kyber::keypair as kyber_keypair;
use kyber::encapsulate as kyber_encapsulate;
use kyber::decapsulate as kyber_decapsulate;
use dilithium::keypair as dilithium_keypair;
use dilithium::sign as dilithium_sign;
use dilithium::verify as dilithium_verify;
use falcon::keypair as falcon_keypair;
use falcon::sign as falcon_sign;
use falcon::verify as falcon_verify;

#[derive(Debug, Error)]
pub enum DIDError {
    #[error("Invalid key")]
    InvalidKey,
    #[error("Signature verification failed")]
    SignatureVerification,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key rotation failed")]
    KeyRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Algorithm {
    Secp256k1,
    RSA,
    ECDSA,
    Kyber,
    Dilithium,
    Falcon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String,
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
    algorithm: Algorithm,
}

impl DID {
    pub fn new(id: String, algorithm: Algorithm) -> Self {
        let (secret_key, public_key) = match algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let secret = SecretKey::new(&mut thread_rng());
                let public = PublicKey::from_secret_key(&secp, &secret);
                (secret.to_bytes().to_vec(), public.serialize().to_vec())
            },
            Algorithm::RSA => {
                let private_key = RSAPrivateKey::new(&mut thread_rng(), 2048)
                    .expect("failed to generate RSA key");
                let public_key = private_key.to_public_key();
                (
                    private_key.to_pkcs1().expect("failed to encode private key"),
                    public_key.to_pkcs1().expect("failed to encode public key")
                )
            },
            Algorithm::ECDSA => {
                let signing_key = SigningKey::random(&mut thread_rng());
                let verifying_key = VerifyingKey::from(&signing_key);
                (
                    signing_key.to_bytes().to_vec(),
                    verifying_key.to_bytes().to_vec()
                )
            },
            Algorithm::Kyber => {
                let (public_key, private_key) = kyber_keypair();
                (private_key, public_key)
            },
            Algorithm::Dilithium => {
                let (public_key, private_key) = dilithium_keypair();
                (private_key, public_key)
            },
            Algorithm::Falcon => {
                let (public_key, private_key) = falcon_keypair();
                (private_key, public_key)
            },
        };

        DID {
            id,
            secret_key,
            public_key,
            algorithm,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, DIDError> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let secret_key = SecretKey::from_slice(&self.secret_key)
                    .map_err(|_| DIDError::InvalidKey)?;
                let msg = secp256k1::Message::from_slice(&Sha256::digest(message))
                    .map_err(|_| DIDError::InvalidKey)?;
                Ok(secp.sign(&msg, &secret_key).serialize_compact().to_vec())
            },
            Algorithm::RSA => {
                let private_key = RSAPrivateKey::from_pkcs1(&self.secret_key).expect("failed to decode private key");
                let padding = rsa::PaddingScheme::new_pkcs1v15_sign(None);
                Ok(private_key.sign(padding, &Sha256::digest(message)).expect("failed to sign message"))
            },
            Algorithm::ECDSA => {
                let signing_key = SigningKey::from_bytes(&self.secret_key).expect("failed to decode private key");
                Ok(signing_key.sign(message).to_bytes().to_vec())
            },
            Algorithm::Dilithium => {
                Ok(dilithium_sign(&self.secret_key, message))
            },
            Algorithm::Falcon => {
                Ok(falcon_sign(&self.secret_key, message))
            },
            _ => Err(DIDError::InvalidKey),
        }
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> Result<bool, DIDError> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let public_key = PublicKey::from_slice(&self.public_key)
                    .map_err(|_| DIDError::InvalidKey)?;
                let msg = secp256k1::Message::from_slice(&Sha256::digest(message))
                    .map_err(|_| DIDError::InvalidKey)?;
                let sig = secp256k1::Signature::from_compact(signature)
                    .map_err(|_| DIDError::SignatureVerification)?;
                Ok(secp.verify(&msg, &sig, &public_key).is_ok())
            },
            Algorithm::RSA => {
                if let Ok(public_key) = RSAPublicKey::from_pkcs1(&self.public_key) {
                    let padding = rsa::PaddingScheme::new_pkcs1v15_sign(None);
                    Ok(public_key.verify(padding, &Sha256::digest(message), signature).is_ok())
                } else {
                    Err(DIDError::InvalidKey)
                }
            },
            Algorithm::ECDSA => {
                if let Ok(verifying_key) = VerifyingKey::from_bytes(&self.public_key) {
                    Ok(verifying_key.verify(message, signature).is_ok())
                } else {
                    Err(DIDError::InvalidKey)
                }
            },
            Algorithm::Dilithium => {
                Ok(dilithium_verify(&self.public_key, message, signature))
            },
            Algorithm::Falcon => {
                Ok(falcon_verify(&self.public_key, message, signature))
            },
            _ => Err(DIDError::InvalidKey),
        }
    }

    pub fn rotate_key(&mut self) -> Result<(), DIDError> {
        let (new_secret_key, new_public_key) = match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let secret = SecretKey::new(&mut thread_rng());
                let public = PublicKey::from_secret_key(&secp, &secret);
                (secret.to_bytes().to_vec(), public.serialize().to_vec())
            },
            Algorithm::RSA => {
                let private_key = RSAPrivateKey::new(&mut thread_rng(), 2048)
                    .expect("failed to generate RSA key");
                let public_key = private_key.to_public_key();
                (
                    private_key.to_pkcs1().expect("failed to encode private key"),
                    public_key.to_pkcs1().expect("failed to encode public key")
                )
            },
            Algorithm::ECDSA => {
                let signing_key = SigningKey::random(&mut thread_rng());
                let verifying_key = VerifyingKey::from(&signing_key);
                (
                    signing_key.to_bytes().to_vec(),
                    verifying_key.to_bytes().to_vec()
                )
            },
            Algorithm::Kyber => {
                let (public_key, private_key) = kyber_keypair();
                (private_key, public_key)
            },
            Algorithm::Dilithium => {
                let (public_key, private_key) = dilithium_keypair();
                (private_key, public_key)
            },
            Algorithm::Falcon => {
                let (public_key, private_key) = falcon_keypair();
                (private_key, public_key)
            },
        };

        self.secret_key = new_secret_key;
        self.public_key = new_public_key;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableDID {
    pub id: String,
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub algorithm: Algorithm,
}

impl From<&DID> for SerializableDID {
    fn from(did: &DID) -> Self {
        SerializableDID {
            id: did.id.clone(),
            secret_key: did.secret_key.clone(),
            public_key: did.public_key.clone(),
            algorithm: did.algorithm.clone(),
        }
    }
}

impl From<&SerializableDID> for DID {
    fn from(serializable_did: &SerializableDID) -> Self {
        DID {
            id: serializable_did.id.clone(),
            secret_key: serializable_did.secret_key.clone(),
            public_key: serializable_did.public_key.clone(),
            algorithm: serializable_did.algorithm.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_support() {
        for alg in &[Algorithm::Secp256k1, Algorithm::RSA, Algorithm::ECDSA, Algorithm::Kyber, Algorithm::Dilithium, Algorithm::Falcon] {
            let did = DID::new("did:example:123".to_string(), alg.clone());
            let message = b"test message";
            let signature = did.sign_message(message).unwrap();
            assert!(did.verify_signature(message, &signature).unwrap());
        }
    }

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
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());
    }

    #[test]
    fn test_secp256k1_signing() {
        let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
        let message = b"test message";
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());
    }

    #[test]
    fn test_rsa_signing() {
        let did = DID::new("did:example:123".to_string(), Algorithm::RSA);
        let message = b"test message";
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());
    }

    #[test]
    fn test_ecdsa_signing() {
        let did = DID::new("did:example:123".to_string(), Algorithm::ECDSA);
        let message = b"test message";
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());
    }

    #[test]
    fn test_dilithium_signing() {
        let did = DID::new("did:example:123".to_string(), Algorithm::Dilithium);
        let message = b"test message";
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());
    }

    #[test]
    fn test_falcon_signing() {
        let did = DID::new("did:example:123".to_string(), Algorithm::Falcon);
        let message = b"test message";
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());
    }

    #[test]
    fn test_serialization() {
        let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
        let serializable: SerializableDID = (&did).into();
        let deserialized: DID = (&serializable).into();
        
        assert_eq!(did.id, deserialized.id);
        assert_eq!(did.public_key, deserialized.public_key);
        assert_eq!(did.algorithm, deserialized.algorithm);
    }

    #[test]
    fn test_key_rotation() {
        let mut did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
        let old_public_key = did.public_key.clone();
        did.rotate_key().unwrap();
        assert_ne!(old_public_key, did.public_key);
    }

    #[test]
    fn test_detailed_validation_error_handling() {
        let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
        let message = b"test message";
        let invalid_signature = vec![0u8; 64];
        let result = did.verify_signature(message, &invalid_signature);
        assert!(matches!(result, Err(DIDError::SignatureVerification)));
    }

    #[test]
    fn test_comprehensive_test_coverage() {
        let did = DID::new("did:example:123".to_string(), Algorithm::Secp256k1);
        let message = b"test message";
        let signature = did.sign_message(message).unwrap();
        assert!(did.verify_signature(message, &signature).unwrap());

        let long_message = vec![0u8; 10000];
        let signature = did.sign_message(&long_message).unwrap();
        assert!(did.verify_signature(&long_message, &signature).unwrap());

        let empty_message = b"";
        let signature = did.sign_message(empty_message).unwrap();
        assert!(did.verify_signature(empty_message, &signature).unwrap());
    }
}
