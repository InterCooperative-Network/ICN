use rsa::{pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey, RsaPrivateKey, RsaPublicKey};
use secp256k1::{Message, Secp256k1, SecretKey, PublicKey};
use sha2::{Sha256, Digest};
use rand::rngs::OsRng;
use ecdsa::{
    SigningKey,
    VerifyingKey,
    signature::{Signer, Verifier},
};
use p256::NistP256;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Algorithm {
    Ed25519,
    Secp256k1,
    RSA,
    // Post-quantum algorithms to be implemented in the future
    // Kyber,
    // Dilithium,
    // Falcon,
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Unsupported algorithm: {0:?}")]
    UnsupportedAlgorithm(Algorithm),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub algorithm: Algorithm,
}

impl KeyPair {
    pub fn generate(algorithm: Algorithm) -> Result<Self, Box<dyn std::error::Error>> {
        match algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let mut rng = OsRng;
                let secret_key = SecretKey::new(&mut rng);
                let public_key = PublicKey::from_secret_key(&secp, &secret_key);
                
                Ok(KeyPair {
                    private_key: secret_key[..].to_vec(),
                    public_key: public_key.serialize().to_vec(),
                    algorithm: Algorithm::Secp256k1,
                })
            }
            Algorithm::RSA => {
                let mut rng = OsRng;
                let bits = 2048;
                let private_key = RsaPrivateKey::new(&mut rng, bits)?;
                let public_key = RsaPublicKey::from(&private_key);
                
                let private_der = private_key.to_pkcs1_der()?;
                let public_der = public_key.to_pkcs1_der()?;
                
                Ok(KeyPair {
                    private_key: private_der.as_bytes().to_vec(),
                    public_key: public_der.as_bytes().to_vec(),
                    algorithm: Algorithm::RSA,
                })
            }
            Algorithm::Ed25519 => {
                let signing_key = SigningKey::<NistP256>::random(&mut OsRng);
                let verifying_key = VerifyingKey::from(&signing_key);
                
                Ok(KeyPair {
                    private_key: signing_key.to_bytes().to_vec(),
                    public_key: verifying_key.to_encoded_point(false).as_bytes().to_vec(),
                    algorithm: Algorithm::Ed25519,
                })
            }
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let secret_key = SecretKey::from_slice(&self.private_key)?;
                let message_hash = Sha256::digest(message);
                let message = Message::from_digest_slice(&message_hash)?;
                let sig = secp.sign_ecdsa(&message, &secret_key);
                Ok(sig.serialize_der().to_vec())
            }
            Algorithm::RSA => {
                let private_key = RsaPrivateKey::from_pkcs1_der(&self.private_key)?;
                let padding = rsa::pkcs1v15::SigningPaddingScheme::new_pkcs1v15_sign(None);
                Ok(private_key.sign(padding, message)?)
            }
            Algorithm::Ed25519 => {
                let signing_key = SigningKey::<NistP256>::from_bytes(&self.private_key.try_into()?)?;
                Ok(signing_key.sign(message).to_vec())
            }
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let public_key = PublicKey::from_slice(&self.public_key)?;
                let message_hash = Sha256::digest(message);
                let message = Message::from_digest_slice(&message_hash)?;
                let sig = secp256k1::ecdsa::Signature::from_der(signature)?;
                Ok(secp.verify_ecdsa(&message, &sig, &public_key).is_ok())
            }
            Algorithm::RSA => {
                let public_key = RsaPublicKey::from_pkcs1_der(&self.public_key)?;
                let padding = rsa::pkcs1v15::SigningPaddingScheme::new_pkcs1v15_sign(None);
                Ok(public_key.verify(padding, message, signature).is_ok())
            }
            Algorithm::Ed25519 => {
                let verifying_key = VerifyingKey::from_encoded_point(
                    &p256::EncodedPoint::from_bytes(&self.public_key)?
                )?;
                Ok(verifying_key.verify(message, &ecdsa::Signature::from_bytes(signature.try_into()?)).is_ok())
            }
        }
    }
}

pub fn hash(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secp256k1() {
        let keypair = KeyPair::generate(Algorithm::Secp256k1).unwrap();
        let message = b"Hello, world!";
        let signature = keypair.sign(message).unwrap();
        let valid = keypair.verify(message, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_rsa() {
        let keypair = KeyPair::generate(Algorithm::RSA).unwrap();
        let message = b"Hello, world!";
        let signature = keypair.sign(message).unwrap();
        let valid = keypair.verify(message, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_ecdsa() {
        let keypair = KeyPair::generate(Algorithm::Ed25519).unwrap();
        let message = b"Hello, world!";
        let signature = keypair.sign(message).unwrap();
        let valid = keypair.verify(message, &signature).unwrap();
        assert!(valid);
    }
}
