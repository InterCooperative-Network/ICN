mod blockchain;
mod did;
mod transaction;
mod reputation;
mod governance;

use blockchain::Blockchain;
use did::DID;
use transaction::Transaction;
use reputation::ReputationSystem;
use governance::Proposal;

fn main() {
    // Initialize Blockchain, Reputation System, and Governance
    let mut blockchain = Blockchain::new();
    let mut reputation_system = ReputationSystem::new();

    // Generate DIDs for testing governance and transactions
    let (sender_did, _) = DID::generate_random(String::from("did:icn:001"));
    let (receiver_did, _) = DID::generate_random(String::from("did:icn:002"));

    // Create transactions and update reputation
    let transaction1 = Transaction::new(sender_did.id.clone(), receiver_did.id.clone(), 100);
    blockchain.add_transaction(transaction1.clone());
    reputation_system.increase_reputation(&sender_did.id, 10);

    let transaction2 = Transaction::new(receiver_did.id.clone(), sender_did.id.clone(), 50);
    blockchain.add_transaction(transaction2.clone());
    reputation_system.decrease_reputation(&receiver_did.id, 5);

    // Mine the block with pending transactions
    blockchain.mine_block();

    // Governance - Create a proposal and conduct voting
    let mut proposal = Proposal::new(1, String::from("Increase community funding"));

    // Display reputation for weighted voting
    let sender_reputation = reputation_system.get_reputation(&sender_did.id);
    let receiver_reputation = reputation_system.get_reputation(&receiver_did.id);

    // Cast votes based on reputation scores
    proposal.vote(&sender_did.id, sender_reputation);
    proposal.vote(&receiver_did.id, receiver_reputation);

    // Close the proposal and display the result
    proposal.close();
    println!(
        "Proposal '{}' Results - Total Weighted Votes: {}",
        proposal.description,
        proposal.total_votes()
    );

    // Display Blockchain and Reputation for Reference
    println!("Blockchain: {:?}", blockchain.chain);
    println!(
        "Reputation Scores:\nSender: {} => {}\nReceiver: {} => {}",
        sender_did.id,
        reputation_system.get_reputation(&sender_did.id),
        receiver_did.id,
        reputation_system.get_reputation(&receiver_did.id)
    );
}
