use secp256k1::{Secp256k1, Message, SecretKey, PublicKey, Signature};
use sha2::{Sha256, Digest};
use thiserror::Error;
use crate::error::{ConsensusError, ConsensusResult};

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Signing error: {0}")]
    SigningError(String),
}

pub type CryptoResult<T> = Result<T, CryptoError>;

/// Handles cryptographic operations for consensus
pub struct CryptoManager {
    secp: Secp256k1<secp256k1::All>,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::new(),
        }
    }

    /// Generate a keypair for a validator
    pub fn generate_keypair(&self) -> CryptoResult<(SecretKey, PublicKey)> {
        let mut rng = rand::thread_rng();
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&self.secp, &secret_key);
        
        Ok((secret_key, public_key))
    }

    /// Sign a message with a secret key
    pub fn sign(&self, message: &[u8], secret_key: &SecretKey) -> CryptoResult<Signature> {
        let message = self.hash_message(message);
        let message = Message::from_slice(&message)
            .map_err(|e| CryptoError::SigningError(e.to_string()))?;
        
        Ok(self.secp.sign_ecdsa(&message, secret_key))
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature, public_key: &PublicKey) -> CryptoResult<bool> {
        let message = self.hash_message(message);
        let message = Message::from_slice(&message)
            .map_err(|e| CryptoError::SigningError(e.to_string()))?;
        
        Ok(self.secp.verify_ecdsa(&message, signature, public_key).is_ok())
    }

    /// Hash a message using SHA256
    fn hash_message(&self, message: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(message);
        hasher.finalize().into()
    }
}

impl CryptoManager {
    /// Integrate cryptographic operations with the consensus engine
    pub fn integrate_with_consensus(&self, consensus: &mut crate::ConsensusEngine) -> ConsensusResult<()> {
        // Example integration logic
        let (secret_key, public_key) = self.generate_keypair()?;
        let message = b"consensus message";
        let signature = self.sign(message, &secret_key)?;
        let is_valid = self.verify(message, &signature, &public_key)?;

        if is_valid {
            consensus.process_signature(public_key, signature)?;
        } else {
            return Err(ConsensusError::InvalidSignature);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConsensusEngine;

    #[test]
    fn test_signature_verification() {
        let crypto = CryptoManager::new();
        let (secret_key, public_key) = crypto.generate_keypair().unwrap();
        
        let message = b"test message";
        let signature = crypto.sign(message, &secret_key).unwrap();
        
        assert!(crypto.verify(message, &signature, &public_key).unwrap());
        
        // Test invalid signature
        let wrong_message = b"wrong message";
        assert!(!crypto.verify(wrong_message, &signature, &public_key).unwrap());
    }

    #[test]
    fn test_integration_with_consensus() {
        let crypto = CryptoManager::new();
        let mut consensus = ConsensusEngine::new().unwrap();
        
        assert!(crypto.integrate_with_consensus(&mut consensus).is_ok());
    }
}
