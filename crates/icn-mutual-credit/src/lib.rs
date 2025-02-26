use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use icn_crypto::KeyPair; // Import KeyPair for signature verification

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MutualCreditTransaction {
    pub sender_did: String,
    pub receiver_did: String,
    pub amount: i64,
    pub signature: String,
    pub timestamp: i64,
}

pub struct MutualCreditLedger {
    // Maps member DID to current credit balance.
    pub balances: HashMap<String, i64>,
    // Ordered list of all mutual credit transactions.
    pub history: Vec<MutualCreditTransaction>,
}

impl MutualCreditLedger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            history: Vec::new(),
        }
    }

    // Process a new mutual credit transaction.
    pub fn process_transaction(&mut self, tx: MutualCreditTransaction) -> Result<(), String> {
        // Verify sender signature
        if !Self::verify_signature(&tx.sender_did, &tx.signature, tx.amount) {
            return Err("Invalid signature".into());
        }
        // Update sender and receiver balances
        *self.balances.entry(tx.sender_did.clone()).or_insert(0) -= tx.amount;
        *self.balances.entry(tx.receiver_did.clone()).or_insert(0) += tx.amount;
        // Append to history
        self.history.push(tx);
        Ok(())
    }

    // Verify signature using icn-crypto
    fn verify_signature(did: &str, signature: &str, amount: i64) -> bool {
        // Retrieve public key from IdentityService (placeholder)
        let public_key = vec![]; // Replace with actual public key retrieval logic
        let key_pair = KeyPair {
            public_key,
            private_key: vec![], // Not needed for verification
            algorithm: icn_crypto::Algorithm::Secp256k1, // Assuming Secp256k1 for this example
        };
        key_pair.verify(amount.to_string().as_bytes(), signature.as_bytes())
    }

    // Get balance summary for a member
    pub fn get_balance(&self, did: &str) -> i64 {
        *self.balances.get(did).unwrap_or(&0)
    }

    // Print simple ledger summary (for logging)
    pub fn print_summary(&self) {
        for (did, balance) in &self.balances {
            println!("Member {}: Balance {}", did, balance);
        }
    }
}
