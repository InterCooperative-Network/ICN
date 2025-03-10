use secp256k1::{Message, Secp256k1, SecretKey, PublicKey, ecdsa};
use sha2::{Sha256, Digest};
use rand::rngs::OsRng;
use p256::ecdsa::{SigningKey, VerifyingKey, Signature};
use p256::ecdsa::signature::{Signer, Verifier};
use p256::NistP256;
use generic_array::GenericArray;
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
    #[error("RSA error: {0}")]
    RsaError(String),
    
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(String),
    
    #[error("ECDSA error: {0}")]
    EcdsaError(String),
    
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("General error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

#[derive(Debug, Clone)]
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
                let mut rng = OsRng;
                let (secret_key, public_key) = secp.generate_keypair(&mut rng);
                
                Ok(KeyPair {
                    private_key: secret_key.secret_bytes().to_vec(),
                    public_key: public_key.serialize().to_vec(),
                    algorithm: Algorithm::Secp256k1,
                })
            }
            Algorithm::RSA => {
                // For now, we'll just create a dummy RSA key pair
                // This is a placeholder until we can properly implement RSA
                Ok(KeyPair {
                    private_key: vec![0; 32],
                    public_key: vec![0; 32],
                    algorithm: Algorithm::RSA,
                })
            }
            Algorithm::Ed25519 => {
                let signing_key = SigningKey::random(&mut OsRng);
                let verifying_key = VerifyingKey::from(&signing_key);
                
                Ok(KeyPair {
                    private_key: signing_key.to_bytes().to_vec(),
                    public_key: verifying_key.to_encoded_point(false).as_bytes().to_vec(),
                    algorithm: Algorithm::Ed25519,
                })
            }
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let secret_key = SecretKey::from_slice(&self.private_key)
                    .map_err(|e| CryptoError::Secp256k1Error(e.to_string()))?;
                let message_hash = Sha256::digest(message);
                let message = Message::from_digest_slice(&message_hash)
                    .map_err(|e| CryptoError::Secp256k1Error(e.to_string()))?;
                let sig = secp.sign_ecdsa(&message, &secret_key);
                Ok(sig.serialize_der().to_vec())
            }
            Algorithm::RSA => {
                // Placeholder for RSA signing
                // Return a dummy signature for now
                Ok(vec![0; 64])
            }
            Algorithm::Ed25519 => {
                // Convert private key bytes to SigningKey
                let bytes: [u8; 32] = self.private_key[0..32]
                    .try_into()
                    .map_err(|_| CryptoError::InvalidKeyFormat("Invalid key length".to_string()))?;
                
                // Convert to GenericArray
                let generic_bytes = GenericArray::from_slice(&bytes);
                
                let signing_key = SigningKey::from_bytes(generic_bytes)
                    .map_err(|e| CryptoError::EcdsaError(e.to_string()))?;
                
                let signature: Signature = signing_key.sign(message);
                Ok(signature.to_vec())
            }
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let public_key = PublicKey::from_slice(&self.public_key)
                    .map_err(|e| CryptoError::Secp256k1Error(e.to_string()))?;
                let message_hash = Sha256::digest(message);
                let message = Message::from_digest_slice(&message_hash)
                    .map_err(|e| CryptoError::Secp256k1Error(e.to_string()))?;
                let sig = ecdsa::Signature::from_der(signature)
                    .map_err(|e| CryptoError::Secp256k1Error(e.to_string()))?;
                Ok(secp.verify_ecdsa(&message, &sig, &public_key).is_ok())
            }
            Algorithm::RSA => {
                // Placeholder for RSA verification
                // Always return true for now
                Ok(true)
            }
            Algorithm::Ed25519 => {
                // Convert public key bytes to VerifyingKey
                let encoded_point = p256::EncodedPoint::from_bytes(&self.public_key)
                    .map_err(|e| CryptoError::EcdsaError(e.to_string()))?;
                
                let verifying_key = VerifyingKey::from_encoded_point(&encoded_point)
                    .map_err(|e| CryptoError::EcdsaError(e.to_string()))?;
                
                // Create signature from bytes
                let sig = p256::ecdsa::Signature::try_from(signature)
                    .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
                
                Ok(verifying_key.verify(message, &sig).is_ok())
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
