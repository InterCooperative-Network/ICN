use secp256k1::{Secp256k1, SecretKey, PublicKey};
use serde::Serialize;

/// Struct representing a decentralized identifier (DID)
#[derive(Debug, Clone)]
pub struct DID {
    pub id: String,
    pub public_key: PublicKey,
}

impl DID {
    /// Creates a new DID with a specified identifier and an existing secret key.
    pub fn new(id: String, secret_key: &SecretKey) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, secret_key);

        DID { id, public_key }
    }

    /// Generates a new random DID and secret key pair.
    pub fn generate_random(id: String) -> (Self, SecretKey) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&rand::random::<[u8; 32]>()).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let did = DID { id, public_key };
        (did, secret_key)
    }
}

// Custom serialization for DID
impl Serialize for DID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DID", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("public_key", &self.public_key.to_string())?;
        state.end()
    }
}