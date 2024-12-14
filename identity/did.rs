pub mod creation {
    use secp256k1::SecretKey;
    use rand::thread_rng;

    pub struct DID {
        pub id: String,
        pub secret_key: SecretKey,
    }

    impl DID {
        pub fn new(id: String) -> Self {
            DID {
                id,
                secret_key: SecretKey::new(&mut thread_rng()),
            }
        }
    }
}

pub mod serialization {
    use super::creation::DID;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct SerializableDID {
        pub id: String,
        pub secret_key: Vec<u8>,
    }

    impl From<&DID> for SerializableDID {
        fn from(did: &DID) -> Self {
            SerializableDID {
                id: did.id.clone(),
                secret_key: did.secret_key[..].to_vec(),
            }
        }
    }

    impl From<&SerializableDID> for DID {
        fn from(serializable_did: &SerializableDID) -> Self {
            DID {
                id: serializable_did.id.clone(),
                secret_key: SecretKey::from_slice(&serializable_did.secret_key).unwrap(),
            }
        }
    }
}

pub mod validation {
    use super::creation::DID;
    use secp256k1::{Secp256k1, Message, Signature};

    impl DID {
        pub fn sign_message(&self, message: &[u8]) -> Signature {
            let secp = Secp256k1::new();
            let msg = Message::from_slice(message).expect("32 bytes");
            secp.sign(&msg, &self.secret_key)
        }

        pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
            let secp = Secp256k1::new();
            let msg = Message::from_slice(message).expect("32 bytes");
            secp.verify(&msg, signature, &self.secret_key.public_key(&secp)).is_ok()
        }
    }
}
