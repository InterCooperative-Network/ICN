use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
use secp256k1::ecdsa::Signature as Secp256k1Signature;
use sha2::{Sha256, Digest};
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::DecodeRsaPrivateKey, pkcs1::DecodeRsaPublicKey};
use rsa::traits::PaddingScheme;
use rsa::pkcs1v15::SigningPaddingScheme;
use ecdsa::SigningKey;
use p256::ecdsa::{signature::{Signer, Verifier}, VerifyingKey};
use p256::ecdsa::Signature as EcdsaSignature;
use rand::rngs::ThreadRng;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Secp256k1,
    RSA,
    ECDSA,
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

pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub algorithm: Algorithm,
}

impl KeyPair {
    pub fn generate(algorithm: Algorithm) -> Result<Self> {
        match algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let mut rng = rand::thread_rng();
                let (private_key, public_key) = secp.generate_keypair(&mut rng);
                Ok(KeyPair {
                    public_key: public_key.serialize().to_vec(),
                    private_key: private_key[..].to_vec(),
                    algorithm,
                })
            },
            Algorithm::RSA => {
                let mut rng = rand::thread_rng();
                let private_key = RsaPrivateKey::new(&mut rng, 2048)
                    .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?;
                let public_key = RsaPublicKey::from(&private_key);
                
                let private_der = private_key.to_pkcs1_der()
                    .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?
                    .to_vec();
                let public_der = public_key.to_pkcs1_der()
                    .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?
                    .to_vec();
                
                Ok(KeyPair {
                    public_key: public_der,
                    private_key: private_der,
                    algorithm,
                })
            },
            Algorithm::ECDSA => {
                let mut rng = rand::thread_rng();
                let signing_key = SigningKey::random(&mut rng);
                let verifying_key = VerifyingKey::from(&signing_key);
                
                Ok(KeyPair {
                    public_key: verifying_key.to_encoded_point(false).as_bytes().to_vec(),
                    private_key: signing_key.to_bytes().to_vec(),
                    algorithm,
                })
            },
            // Post-quantum algorithms to be implemented in the future
            // Algorithm::Kyber => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
            // Algorithm::Dilithium => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
            // Algorithm::Falcon => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let private_key = SecretKey::from_slice(&self.private_key)
                    .map_err(|_| CryptoError::InvalidKey("Invalid Secp256k1 private key".to_string()))?;
                let message_hash = Sha256::digest(message);
                let message = Message::from_slice(&message_hash)
                    .map_err(|_| CryptoError::SigningFailed("Failed to create message".to_string()))?;
                
                Ok(secp.sign_ecdsa(&message, &private_key).serialize_compact().to_vec())
            },
            Algorithm::RSA => {
                let private_key = RsaPrivateKey::from_pkcs1_der(&self.private_key)
                    .map_err(|e| CryptoError::InvalidKey(format!("Failed to decode RSA private key: {}", e)))?;
                
                let padding = SigningPaddingScheme::new_pkcs1v15_sign(None);
                let signature = private_key.sign(padding, &Sha256::digest(message))
                    .map_err(|e| CryptoError::SigningFailed(format!("RSA signing failed: {}", e)))?;
                
                Ok(signature)
            },
            Algorithm::ECDSA => {
                let signing_key = SigningKey::from_bytes(&self.private_key)
                    .map_err(|_| CryptoError::InvalidKey("Invalid ECDSA private key".to_string()))?;
                
                let signature: EcdsaSignature = signing_key.sign(message);
                Ok(signature.to_vec())
            },
            // Post-quantum algorithms to be implemented in the future
            // Algorithm::Kyber => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
            // Algorithm::Dilithium => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
            // Algorithm::Falcon => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let public_key = PublicKey::from_slice(&self.public_key)
                    .map_err(|_| CryptoError::InvalidKey("Invalid Secp256k1 public key".to_string()))?;
                let message_hash = Sha256::digest(message);
                let message = Message::from_slice(&message_hash)
                    .map_err(|_| CryptoError::VerificationFailed("Failed to create message".to_string()))?;
                let signature = Secp256k1Signature::from_compact(signature)
                    .map_err(|_| CryptoError::InvalidSignature("Invalid Secp256k1 signature".to_string()))?;
                
                match secp.verify_ecdsa(&message, &signature, &public_key) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            },
            Algorithm::RSA => {
                let public_key = RsaPublicKey::from_pkcs1_der(&self.public_key)
                    .map_err(|e| CryptoError::InvalidKey(format!("Failed to decode RSA public key: {}", e)))?;
                
                let padding = SigningPaddingScheme::new_pkcs1v15_sign(None);
                let result = public_key.verify(padding, &Sha256::digest(message), signature);
                
                Ok(result.is_ok())
            },
            Algorithm::ECDSA => {
                let verifying_key = VerifyingKey::from_encoded_point(
                    &p256::EncodedPoint::from_bytes(&self.public_key)
                        .map_err(|_| CryptoError::InvalidKey("Invalid ECDSA public key".to_string()))?
                ).map_err(|_| CryptoError::InvalidKey("Invalid ECDSA public key format".to_string()))?;
                
                let signature_bytes = EcdsaSignature::try_from(signature)
                    .map_err(|_| CryptoError::InvalidSignature("Invalid ECDSA signature".to_string()))?;
                
                match verifying_key.verify(message, &signature_bytes) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            },
            // Post-quantum algorithms to be implemented in the future
            // Algorithm::Kyber => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
            // Algorithm::Dilithium => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
            // Algorithm::Falcon => Err(CryptoError::UnsupportedAlgorithm(algorithm)),
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
        let keypair = KeyPair::generate(Algorithm::ECDSA).unwrap();
        let message = b"Hello, world!";
        let signature = keypair.sign(message).unwrap();
        let valid = keypair.verify(message, &signature).unwrap();
        assert!(valid);
    }
}
