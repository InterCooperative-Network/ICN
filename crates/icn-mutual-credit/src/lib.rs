use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};

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
        // Verify sender signature (placeholder)
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

    // Dummy signature verifier (to be replaced with actual verification logic)
    fn verify_signature(_did: &str, _signature: &str, _amount: i64) -> bool {
        true // In practice, verify using the sender's DID key.
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

// Example usage (to be removed or used in unit tests)
// fn main() {
//     let mut ledger = MutualCreditLedger::new();
//     let tx = MutualCreditTransaction {
//         sender_did: "did:icn:member1".into(),
//         receiver_did: "did:icn:member2".into(),
//         amount: 50,
//         signature: "signature_placeholder".into(),
//         timestamp: Utc::now().timestamp(),
//     };
//     ledger.process_transaction(tx).unwrap();
//     ledger.print_summary();
// }
