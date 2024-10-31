mod blockchain;
mod did;
mod transaction;
mod reputation;

use blockchain::Block;
use did::DID;
use transaction::Transaction;
use reputation::ReputationSystem;

fn main() {
    // Generate the Genesis Block
    let genesis_block = Block::new(0, String::from("0"), String::from("Genesis Block"));
    println!("Genesis Block: {:?}", genesis_block);

    // Generate two DIDs for transaction and reputation demonstration
    let (sender_did, _) = DID::generate_random(String::from("did:icn:001"));
    let (receiver_did, _) = DID::generate_random(String::from("did:icn:002"));

    // Create a sample transaction
    let transaction = Transaction::new(sender_did.id.clone(), receiver_did.id.clone(), 100);
    println!("New Transaction: {:?}", transaction);

    // Initialize the reputation system and modify reputation
    let mut reputation_system = ReputationSystem::new();
    reputation_system.increase_reputation(&sender_did.id, 10);  // Reward sender for the transaction
    reputation_system.decrease_reputation(&receiver_did.id, 5);  // Penalize receiver (example case)

    // Display reputation scores
    println!(
        "Reputation Scores:\nSender: {} => {}\nReceiver: {} => {}",
        sender_did.id,
        reputation_system.get_reputation(&sender_did.id),
        receiver_did.id,
        reputation_system.get_reputation(&receiver_did.id)
    );
}
