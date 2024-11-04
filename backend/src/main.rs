mod blockchain;
mod identity;
mod reputation;
mod governance;
mod utils;
mod vm;

use std::collections::HashMap;
use blockchain::{Blockchain, transaction::{Transaction, TransactionType}};
use identity::{DID, IdentitySystem};
use reputation::ReputationSystem;
use governance::{Proposal, ProposalType, ProposalHistory};
use vm::{Contract, CooperativeMetadata, OpCode, ResourceImpact};

fn main() {
    // Initialize systems
    let mut identity_system = IdentitySystem::new();
    let mut reputation_system = ReputationSystem::new();

    // Initialize Blockchain with IdentitySystem and ReputationSystem
    let mut blockchain = Blockchain::new(identity_system.clone(), reputation_system.clone());
    let mut proposal_history = ProposalHistory::new();

    // Generate DIDs
    let (sender_did, _) = DID::generate_random(String::from("did:icn:001"));
    let (receiver_did, _) = DID::generate_random(String::from("did:icn:002"));

    // Register DIDs in IdentitySystem with permissions
    identity_system.register_did(
        sender_did.clone(),
        vec![
            "cooperative.create".to_string(),
            "proposal.create".to_string(),
            "resource.allocate".to_string()
        ]
    );
    identity_system.register_did(receiver_did.clone(), vec![]);

    // Set initial reputation for sender (important for contract execution)
    reputation_system.increase_reputation(&sender_did.id, 100);  // Ensure enough reputation for contract execution

    // Create transactions
    let transaction1 = Transaction::new(
        sender_did.id.clone(),
        TransactionType::Transfer {
            receiver: receiver_did.id.clone(),
            amount: 100,
        },
    );
    blockchain.add_transaction(transaction1);

    let transaction2 = Transaction::new(
        receiver_did.id.clone(),
        TransactionType::Transfer {
            receiver: sender_did.id.clone(),
            amount: 50,
        },
    );
    blockchain.add_transaction(transaction2);

    // Finalize the block
    blockchain.finalize_block();

    // Process transactions
    let transactions_to_process: Vec<Transaction> = blockchain.chain.last()
        .map(|block| block.transactions.clone())
        .unwrap_or_default();
    
    for transaction in transactions_to_process {
        blockchain.process_transaction(&transaction)
            .unwrap_or_else(|e| println!("{}", e));
    }

    // Create and manage proposals
    let proposal1 = Proposal::new(
        1,
        ProposalType::Funding,
        String::from("Proposal for funding development."),
    );
    proposal_history.add_proposal(proposal1);

    let proposal2 = Proposal::new(
        2,
        ProposalType::PolicyChange,
        String::from("Proposal for changing cooperative policy."),
    );
    proposal_history.add_proposal(proposal2);

    // Vote and manage proposals
    proposal_history.send_voting_reminder();
    if let Some(proposal) = proposal_history.proposals.get_mut(0) {
        proposal.vote(&sender_did.id, 10);
        proposal.close();
    }
    if let Some(proposal) = proposal_history.proposals.get_mut(1) {
        proposal.vote(&receiver_did.id, 5);
    }

    // Display proposal history
    proposal_history.display_history();

    // Create and setup a test contract
    let metadata = CooperativeMetadata {
        creator_did: sender_did.id.clone(),
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

    let contract = Contract {
        id: "contract1".to_string(),
        code: vec![
            OpCode::Push(1),
            OpCode::AllocateResource,
            OpCode::Halt,
        ],
        state: HashMap::new(),
        required_reputation: 50,  // This should now be less than sender's reputation
        cooperative_metadata: metadata.clone(),
        version: "1.0.0".to_string(),
        dependencies: Vec::new(),
        permissions: vec!["resource.allocate".to_string()],
    };

    // Register contract and create transaction
    blockchain.contracts.insert(contract.id.clone(), contract.clone());

    let contract_transaction = Transaction::new(
        sender_did.id.clone(),
        TransactionType::ContractExecution {
            contract_id: contract.id.clone(),
            input_data: HashMap::new(),
        },
    );

    blockchain.add_transaction(contract_transaction);
    blockchain.finalize_block();

    // Process the new block's transactions
    let transactions_to_process: Vec<Transaction> = blockchain.chain.last()
        .map(|block| block.transactions.clone())
        .unwrap_or_default();
    
    for transaction in transactions_to_process {
        blockchain.process_transaction(&transaction)
            .unwrap_or_else(|e| println!("{}", e));
    }

    println!("Blockchain and VM integration completed.");
}