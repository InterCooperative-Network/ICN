pub mod permission_handling {
    use std::collections::HashMap;

    pub struct IdentitySystem {
        permissions: HashMap<String, Vec<String>>,
    }

    impl IdentitySystem {
        pub fn new() -> Self {
            IdentitySystem {
                permissions: HashMap::new(),
            }
        }

        pub fn register_did(&mut self, did: String, permissions: Vec<String>) {
            self.permissions.insert(did, permissions);
        }

        pub fn has_permission(&self, did: &str, permission: &str) -> bool {
            if let Some(perms) = self.permissions.get(did) {
                perms.contains(&permission.to_string())
            } else {
                false
            }
        }
    }
}

pub mod role_management {
    use std::collections::HashMap;

    pub struct IdentitySystem {
        roles: HashMap<String, Vec<String>>,
    }

    impl IdentitySystem {
        pub fn new() -> Self {
            IdentitySystem {
                roles: HashMap::new(),
            }
        }

        pub fn assign_role(&mut self, did: String, role: String) {
            self.roles.entry(did).or_insert_with(Vec::new).push(role);
        }

        pub fn get_roles(&self, did: &str) -> Vec<String> {
            self.roles.get(did).cloned().unwrap_or_default()
        }
    }
}

pub mod identity_verification {
    use secp256k1::{Secp256k1, PublicKey, SecretKey, Message, Signature};
    use std::collections::HashMap;

    pub struct IdentitySystem {
        public_keys: HashMap<String, PublicKey>,
    }

    impl IdentitySystem {
        pub fn new() -> Self {
            IdentitySystem {
                public_keys: HashMap::new(),
            }
        }

        pub fn register_did(&mut self, did: String, public_key: PublicKey) {
            self.public_keys.insert(did, public_key);
        }

        pub fn verify_did(&self, did: &str, message: &[u8], signature: &Signature) -> bool {
            if let Some(public_key) = self.public_keys.get(did) {
                let secp = Secp256k1::new();
                let msg = Message::from_slice(message).expect("32 bytes");
                secp.verify(&msg, signature, public_key).is_ok()
            } else {
                false
            }
        }
    }
}
