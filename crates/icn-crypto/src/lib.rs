use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, Signature};
use sha2::{Sha256, Digest};
use rsa::{RSAPrivateKey, RSAPublicKey, PaddingScheme};
use ecdsa::{SigningKey, VerifyingKey, signature::Signer, signature::Verifier};
use icn_types::Algorithm;
use kyber::keypair as kyber_keypair;
use kyber::encapsulate as kyber_encapsulate;
use kyber::decapsulate as kyber_decapsulate;
use dilithium::keypair as dilithium_keypair;
use dilithium::sign as dilithium_sign;
use dilithium::verify as dilithium_verify;
use falcon::keypair as falcon_keypair;
use falcon::sign as falcon_sign;
use falcon::verify as falcon_verify;

pub enum Algorithm {
    Secp256k1,
    RSA,
    ECDSA,
    Kyber,
    Dilithium,
    Falcon,
}

pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub algorithm: Algorithm,
}

impl KeyPair {
    pub fn generate(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let (private_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
                KeyPair {
                    public_key: public_key.serialize().to_vec(),
                    private_key: private_key[..].to_vec(),
                    algorithm,
                }
            },
            Algorithm::RSA => {
                let private_key = RSAPrivateKey::new(&mut rand::thread_rng(), 2048).expect("failed to generate a key");
                let public_key = RSAPublicKey::from(&private_key);
                KeyPair {
                    public_key: public_key.to_pkcs1().expect("failed to encode public key"),
                    private_key: private_key.to_pkcs1().expect("failed to encode private key"),
                    algorithm,
                }
            },
            Algorithm::ECDSA => {
                let signing_key = SigningKey::random(&mut rand::thread_rng());
                let verifying_key = VerifyingKey::from(&signing_key);
                KeyPair {
                    public_key: verifying_key.to_bytes().to_vec(),
                    private_key: signing_key.to_bytes().to_vec(),
                    algorithm,
                }
            },
            Algorithm::Kyber => {
                let (public_key, private_key) = kyber_keypair();
                KeyPair {
                    public_key,
                    private_key,
                    algorithm,
                }
            },
            Algorithm::Dilithium => {
                let (public_key, private_key) = dilithium_keypair();
                KeyPair {
                    public_key,
                    private_key,
                    algorithm,
                }
            },
            Algorithm::Falcon => {
                let (public_key, private_key) = falcon_keypair();
                KeyPair {
                    public_key,
                    private_key,
                    algorithm,
                }
            },
        }
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let private_key = SecretKey::from_slice(&self.private_key).expect("32 bytes");
                let message = Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
                secp.sign(&message, &private_key).serialize_compact().to_vec()
            },
            Algorithm::RSA => {
                let private_key = RSAPrivateKey::from_pkcs1(&self.private_key).expect("failed to decode private key");
                let padding = PaddingScheme::new_pkcs1v15_sign(None);
                private_key.sign(padding, &Sha256::digest(message)).expect("failed to sign message")
            },
            Algorithm::ECDSA => {
                let signing_key = SigningKey::from_bytes(&self.private_key).expect("failed to decode private key");
                signing_key.sign(message).to_bytes().to_vec()
            },
            Algorithm::Dilithium => {
                dilithium_sign(&self.private_key, message)
            },
            Algorithm::Falcon => {
                falcon_sign(&self.private_key, message)
            },
            _ => vec![],
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let public_key = PublicKey::from_slice(&self.public_key).expect("invalid public key");
                let message = Message::from_slice(&Sha256::digest(message)).expect("32 bytes");
                let signature = Signature::from_compact(signature).expect("invalid signature");
                secp.verify(&message, &signature, &public_key).is_ok()
            },
            Algorithm::RSA => {
                let public_key = RSAPublicKey::from_pkcs1(&self.public_key).expect("failed to decode public key");
                let padding = PaddingScheme::new_pkcs1v15_sign(None);
                public_key.verify(padding, &Sha256::digest(message), signature).is_ok()
            },
            Algorithm::ECDSA => {
                let verifying_key = VerifyingKey::from_bytes(&self.public_key).expect("failed to decode public key");
                verifying_key.verify(message, signature).is_ok()
            },
            Algorithm::Dilithium => {
                dilithium_verify(&self.public_key, message, signature)
            },
            Algorithm::Falcon => {
                falcon_verify(&self.public_key, message, signature)
            },
            _ => false,
        }
    }
}

pub fn hash(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}
