pub mod creation {
    use secp256k1::SecretKey;
    use rand::thread_rng;
    use crate::Algorithm;

    pub struct DID {
        pub id: String,
        pub secret_key: SecretKey,
        pub algorithm: Algorithm,
    }

    impl DID {
        pub fn new(id: String, algorithm: Algorithm) -> Self {
            DID {
                id,
                secret_key: SecretKey::new(&mut thread_rng()),
                algorithm,
            }
        }
    }
}

pub mod serialization {
    use super::creation::DID;
    use serde::{Serialize, Deserialize};
    use crate::Algorithm;

    #[derive(Serialize, Deserialize)]
    pub struct SerializableDID {
        pub id: String,
        pub secret_key: Vec<u8>,
        pub algorithm: Algorithm,
    }

    impl From<&DID> for SerializableDID {
        fn from(did: &DID) -> Self {
            SerializableDID {
                id: did.id.clone(),
                secret_key: did.secret_key[..].to_vec(),
                algorithm: did.algorithm.clone(),
            }
        }
    }

    impl From<&SerializableDID> for DID {
        fn from(serializable_did: &SerializableDID) -> Self {
            DID {
                id: serializable_did.id.clone(),
                secret_key: SecretKey::from_slice(&serializable_did.secret_key).unwrap(),
                algorithm: serializable_did.algorithm.clone(),
            }
        }
    }
}

pub mod validation {
    use super::creation::DID;
    use secp256k1::{Secp256k1, Message, Signature};
    use crate::Algorithm;

    impl DID {
        pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
            match self.algorithm {
                Algorithm::Secp256k1 => {
                    let secp = Secp256k1::new();
                    let msg = Message::from_slice(message).expect("32 bytes");
                    secp.sign(&msg, &self.secret_key).serialize_compact().to_vec()
                }
                Algorithm::RSA => {
                    // RSA signing logic
                    vec![]
                }
                Algorithm::ECDSA => {
                    // ECDSA signing logic
                    vec![]
                }
            }
        }

        pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
            match self.algorithm {
                Algorithm::Secp256k1 => {
                    let secp = Secp256k1::new();
                    let msg = Message::from_slice(message).expect("32 bytes");
                    let sig = Signature::from_compact(signature).expect("valid signature");
                    secp.verify(&msg, &sig, &self.secret_key.public_key(&secp)).is_ok()
                }
                Algorithm::RSA => {
                    // RSA verification logic
                    false
                }
                Algorithm::ECDSA => {
                    // ECDSA verification logic
                    false
                }
            }
        }
    }
}
