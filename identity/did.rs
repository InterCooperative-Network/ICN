pub mod creation {
    use secp256k1::SecretKey;
    use rsa::{RSAPrivateKey, RSAPublicKey};
    use ecdsa::{SigningKey, VerifyingKey};
    use rand::thread_rng;

    pub enum Algorithm {
        Secp256k1,
        RSA,
        ECDSA,
    }

    pub struct DID {
        pub id: String,
        pub secret_key: Vec<u8>,
        pub algorithm: Algorithm,
    }

    impl DID {
        pub fn new(id: String, algorithm: Algorithm) -> Self {
            let secret_key = match algorithm {
                Algorithm::Secp256k1 => SecretKey::new(&mut thread_rng()).to_bytes().to_vec(),
                Algorithm::RSA => RSAPrivateKey::new(&mut thread_rng(), 2048).expect("failed to generate a key").to_pkcs1().expect("failed to encode private key"),
                Algorithm::ECDSA => SigningKey::random(&mut thread_rng()).to_bytes().to_vec(),
            };

            DID {
                id,
                secret_key,
                algorithm,
            }
        }
    }
}

pub mod serialization {
    use super::creation::{DID, Algorithm};
    use serde::{Serialize, Deserialize};

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
                secret_key: did.secret_key.clone(),
                algorithm: did.algorithm.clone(),
            }
        }
    }

    impl From<&SerializableDID> for DID {
        fn from(serializable_did: &SerializableDID) -> Self {
            DID {
                id: serializable_did.id.clone(),
                secret_key: serializable_did.secret_key.clone(),
                algorithm: serializable_did.algorithm.clone(),
            }
        }
    }
}

pub mod validation {
    use super::creation::{DID, Algorithm};
    use secp256k1::{Secp256k1, Message, Signature};
    use rsa::{RSAPrivateKey, RSAPublicKey, PaddingScheme};
    use ecdsa::{SigningKey, VerifyingKey, signature::Signer, signature::Verifier};
    use sha2::{Sha256, Digest};

    impl DID {
        pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
            match self.algorithm {
                Algorithm::Secp256k1 => {
                    let secp = Secp256k1::new();
                    let private_key = SecretKey::from_slice(&self.secret_key).expect("32 bytes");
                    let msg = Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
                    secp.sign(&msg, &private_key).serialize_compact().to_vec()
                },
                Algorithm::RSA => {
                    let private_key = RSAPrivateKey::from_pkcs1(&self.secret_key).expect("failed to decode private key");
                    let padding = PaddingScheme::new_pkcs1v15_sign(None);
                    private_key.sign(padding, &Sha256::digest(message)).expect("failed to sign message")
                },
                Algorithm::ECDSA => {
                    let signing_key = SigningKey::from_bytes(&self.secret_key).expect("failed to decode private key");
                    signing_key.sign(message).to_bytes().to_vec()
                },
            }
        }

        pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
            match self.algorithm {
                Algorithm::Secp256k1 => {
                    let secp = Secp256k1::new();
                    let public_key = PublicKey::from_slice(&self.secret_key).expect("invalid public key");
                    let msg = Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
                    let signature = Signature::from_compact(signature).expect("invalid signature");
                    secp.verify(&msg, &signature, &public_key).is_ok()
                },
                Algorithm::RSA => {
                    let public_key = RSAPublicKey::from_pkcs1(&self.secret_key).expect("failed to decode public key");
                    let padding = PaddingScheme::new_pkcs1v15_sign(None);
                    public_key.verify(padding, &Sha256::digest(message), signature).is_ok()
                },
                Algorithm::ECDSA => {
                    let verifying_key = VerifyingKey::from_bytes(&self.secret_key).expect("failed to decode public key");
                    verifying_key.verify(message, signature).is_ok()
                },
            }
        }
    }
}
