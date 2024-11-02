mod blockchain;
mod identity;
mod reputation;
mod governance;
mod utils;
mod vm;

use std::collections::HashMap;
use blockchain::{Blockchain, transaction::Transaction};
use identity::DID;
use reputation::ReputationSystem;
use governance::{Proposal, ProposalType, ProposalHistory};
use vm::{VM, Contract, CooperativeMetadata, OpCode, ResourceImpact};

fn main() {
    let mut blockchain = Blockchain::new();
    let mut reputation_system = ReputationSystem::new();
    let mut proposal_history = ProposalHistory::new();

    let (sender_did, _) = DID::generate_random(String::from("did:icn:001"));
    let (receiver_did, _) = DID::generate_random(String::from("did:icn:002"));

    // Create and add transactions
    let transaction1 = Transaction::new(sender_did.id.clone(), receiver_did.id.clone(), 100);
    blockchain.add_transaction(transaction1.clone());
    reputation_system.increase_reputation(&sender_did.id, 10);

    let transaction2 = Transaction::new(receiver_did.id.clone(), sender_did.id.clone(), 50);
    blockchain.add_transaction(transaction2.clone());
    reputation_system.decrease_reputation(&receiver_did.id, 5);

    // Finalize the block
    blockchain.finalize_block();

    // Create proposals for governance
    let proposal1 = Proposal::new(1, ProposalType::Funding, String::from("Proposal for funding development."));
    proposal_history.add_proposal(proposal1);

    let proposal2 = Proposal::new(2, ProposalType::PolicyChange, String::from("Proposal for changing cooperative policy."));
    proposal_history.add_proposal(proposal2);

    // Vote on proposals
    proposal_history.send_voting_reminder();
    proposal_history.proposals[0].vote(&sender_did.id, 10);
    proposal_history.proposals[1].vote(&receiver_did.id, 5);
    proposal_history.proposals[0].close();

    // Display proposal history
    proposal_history.display_history();

    // Create and execute a contract with the new opcode
    let metadata = CooperativeMetadata {
        creator_did: "did:icn:creator".to_string(),
        cooperative_id: "coop1".to_string(),
        purpose: "Allocate resources for development".to_string(),
        resource_impact: ResourceImpact {
            cpu_intensity: 10,
            memory_usage: 10,
            network_usage: 10,
            storage_usage: 10,
            bandwidth_usage: 10,
        },
        federation_id: None,
        creation_timestamp: 1635724800,
        last_updated: 1635724800,
        member_count: 1,
        resource_allocation: HashMap::new(),
    };

    let _contract = Contract {
        code: vec![
            (OpCode::AllocateResource, Some(1)),
            (OpCode::CalculateVotingWeight, None),
            (OpCode::Return, None),
        ],
        state: HashMap::new(),
        required_reputation: 50,
        cooperative_metadata: metadata.clone(),
        version: "1.0.0".to_string(),
        dependencies: Vec::new(),
        permissions: vec!["resource.allocate".to_string(), "voting.calculate".to_string()],
    };

    let mut vm = VM::new(100, reputation_system.scores.clone());
    let execution_context = vm::ExecutionContext {
        caller_did: "did:icn:creator".to_string(),
        cooperative_id: "coop1".to_string(),
        block_number: 1,
        timestamp: 1635724800,
        federation_context: None,
    };
    vm.set_execution_context(execution_context);
    
    let result = vm.execute_instruction(&OpCode::AllocateResource, Some(1), &metadata);
    assert!(result.is_ok());
    println!("VM executed contract successfully.");
}