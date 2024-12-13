use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::thread_rng;
use crate::crypto::CryptoError;
use crate::crypto::CryptoResult;

/// Generates a keypair for a validator
pub fn generate_keypair() -> CryptoResult<(SecretKey, PublicKey)> {
    let secp = Secp256k1::new();
    let mut rng = thread_rng();
    let secret_key = SecretKey::new(&mut rng);
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    
    Ok((secret_key, public_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let result = generate_keypair();
        assert!(result.is_ok());
        let (secret_key, public_key) = result.unwrap();
        assert_eq!(secret_key.len(), 32);
        assert_eq!(public_key.len(), 33);
    }
}
