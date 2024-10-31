mod blockchain;
mod did;
mod transaction;
mod reputation;
mod governance;

use blockchain::Blockchain;
use did::DID;
use transaction::Transaction;
use reputation::ReputationSystem;
use governance::{Proposal, ProposalType, ProposalStatus, ProposalHistory};
use std::collections::VecDeque;

fn main() {
    let mut blockchain = Blockchain::new();
    let mut reputation_system = ReputationSystem::new();
    let mut proposal_history = ProposalHistory::new();

    let (sender_did, _) = DID::generate_random(String::from("did:icn:001"));
    let (receiver_did, _) = DID::generate_random(String::from("did:icn:002"));

    let transaction1 = Transaction::new(sender_did.id.clone(), receiver_did.id.clone(), 100);
    blockchain.add_transaction(transaction1.clone());
    reputation_system.increase_reputation(&sender_did.id, 10);

    let transaction2 = Transaction::new(receiver_did.id.clone(), sender_did.id.clone(), 50);
    blockchain.add_transaction(transaction2.clone());
    reputation_system.decrease_reputation(&receiver_did.id, 5);

    blockchain.mine_block();

    let mut funding_proposal = Proposal::new(
        1,
        String::from("Increase community funding"),
        ProposalType::Funding,
        None,
        60,
    );

    let mut policy_proposal = Proposal::new(
        2,
        String::from("Amend community policy"),
        ProposalType::PolicyChange,
        None,
        60,
    );

    let mut allocation_proposal = Proposal::new(
        3,
        String::from("Allocate 500 units for community project"),
        ProposalType::ResourceAllocation,
        Some(500),
        60,
    );

    let sender_reputation = reputation_system.get_reputation(&sender_did.id);
    let receiver_reputation = reputation_system.get_reputation(&receiver_did.id);

    if funding_proposal.validate(ProposalType::Funding) {
        funding_proposal.vote(&sender_did.id, sender_reputation);
        reputation_system.reward_voting(&sender_did.id, 2);
    }

    if policy_proposal.validate(ProposalType::PolicyChange) {
        policy_proposal.vote(&receiver_did.id, receiver_reputation);
        reputation_system.reward_voting(&receiver_did.id, 2);
    }

    if allocation_proposal.validate(ProposalType::ResourceAllocation) {
        allocation_proposal.vote(&sender_did.id, sender_reputation);
        allocation_proposal.vote(&receiver_did.id, receiver_reputation);
        reputation_system.reward_voting(&sender_did.id, 2);
        reputation_system.reward_voting(&receiver_did.id, 2);
    }

    funding_proposal.check_and_notify(15);
    policy_proposal.check_and_notify(15);
    allocation_proposal.check_and_notify(15);

    funding_proposal.close();
    policy_proposal.close();
    allocation_proposal.check_status();

    proposal_history.add_proposal(funding_proposal);
    proposal_history.add_proposal(policy_proposal);
    proposal_history.add_proposal(allocation_proposal);

    println!("\n=== Proposal History ===");
    proposal_history.display_history();

    println!("\n=== Proposal Analytics ===");
    proposal_history.total_votes_for_proposals();
    proposal_history.participation_rate(100); // Assume a total of 100 members for example
    proposal_history.voting_trends();

    println!("Blockchain: {:?}", blockchain.chain);
    println!(
        "Reputation Scores:\nSender: {} => {}\nReceiver: {} => {}",
        sender_did.id,
        reputation_system.get_reputation(&sender_did.id),
        receiver_did.id,
        reputation_system.get_reputation(&receiver_did.id)
    );
}
