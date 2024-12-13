use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::{thread_rng, RngCore};

pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();
        
        // Generate random bytes for the secret key
        let mut secret_key_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_key_bytes);
        
        // Create secret key from the random bytes
        let private_key = SecretKey::from_slice(&secret_key_bytes)
            .expect("Random bytes should produce valid key");
            
        let public_key = PublicKey::from_secret_key(&secp, &private_key);
        
        KeyPair { public_key, private_key }
    }
}
