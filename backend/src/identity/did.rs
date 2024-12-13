use secp256k1::{Secp256k1, SecretKey, PublicKey};
use serde::{Serialize, Deserialize};
use rand::{thread_rng, RngCore};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String,
    #[serde(serialize_with = "serialize_public_key")]
    #[serde(deserialize_with = "deserialize_public_key")]
    pub public_key: PublicKey,
}

impl DID {
    pub fn new(id: String, secret_key: &SecretKey) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, secret_key);
        DID { id, public_key }
    }

    pub fn generate_random(id: String) -> (Self, SecretKey) {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();
        
        // Generate random bytes for the secret key
        let mut secret_key_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_key_bytes);
        
        // Create secret key from the random bytes
        let secret_key = SecretKey::from_slice(&secret_key_bytes)
            .expect("Random bytes should produce valid key");
            
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let did = DID { id, public_key };
        (did, secret_key)
    }
}

fn serialize_public_key<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&key.to_string())
}

fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let key_str = String::deserialize(deserializer)?;
    PublicKey::from_str(&key_str).map_err(serde::de::Error::custom)
}
