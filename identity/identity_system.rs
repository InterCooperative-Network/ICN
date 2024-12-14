pub struct IdentitySystem {
    permissions: std::collections::HashMap<String, Vec<String>>,
    roles: std::collections::HashMap<String, Vec<String>>,
    public_keys: std::collections::HashMap<String, secp256k1::PublicKey>,
}

impl IdentitySystem {
    pub fn new() -> Self {
        IdentitySystem {
            permissions: std::collections::HashMap::new(),
            roles: std::collections::HashMap::new(),
            public_keys: std::collections::HashMap::new(),
        }
    }

    pub fn register_did(&mut self, did: String, permissions: Vec<String>) {
        self.permissions.insert(did.clone(), permissions);
    }

    pub fn has_permission(&self, did: &str, permission: &str) -> bool {
        if let Some(perms) = self.permissions.get(did) {
            perms.contains(&permission.to_string())
        } else {
            false
        }
    }

    pub fn assign_role(&mut self, did: String, role: String) {
        self.roles.entry(did).or_insert_with(Vec::new).push(role);
    }

    pub fn get_roles(&self, did: &str) -> Vec<String> {
        self.roles.get(did).cloned().unwrap_or_default()
    }

    pub fn register_public_key(&mut self, did: String, public_key: secp256k1::PublicKey) {
        self.public_keys.insert(did, public_key);
    }

    pub fn verify_did(&self, did: &str, message: &[u8], signature: &secp256k1::Signature) -> bool {
        if let Some(public_key) = self.public_keys.get(did) {
            let secp = secp256k1::Secp256k1::new();
            let msg = secp256k1::Message::from_slice(message).expect("32 bytes");
            secp.verify(&msg, signature, public_key).is_ok()
        } else {
            false
        }
    }
}
