use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, Signature};
use sha2::{Sha256, Digest};

pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let (private_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        KeyPair { public_key, private_key }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let secp = Secp256k1::new();
        let message = Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
        secp.sign(&message, &self.private_key)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        let secp = Secp256k1::new();
        let message = Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
        secp.verify(&message, signature, &self.public_key).is_ok()
    }
}

pub fn hash(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}
