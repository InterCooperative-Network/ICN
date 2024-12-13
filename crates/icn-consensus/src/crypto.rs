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
    fn test_integration_with_consensus() {
        let crypto = CryptoManager::new();
        let mut consensus = ConsensusEngine::new().unwrap();
        
        assert!(crypto.integrate_with_consensus(&mut consensus).is_ok());
    }

}
