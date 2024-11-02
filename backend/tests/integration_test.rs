use std::collections::HashMap;
use blockchain::{Blockchain, transaction::Transaction};
use identity::DID;
use reputation::ReputationSystem;
use governance::{Proposal, ProposalType, ProposalHistory};
use vm::{VM, Contract, CooperativeMetadata, OpCode, ResourceImpact};

#[test]
fn test_vm_allocation_and_governance_integration() {
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

    // Create and execute a contract
    let metadata = CooperativeMetadata {
        creator_did: "did:icn:creator".to_string(),
        cooperative_id: "coop1".to_string(),
        purpose: "Allocate resources for development".to_string(),
        resource_impact: ResourceImpact {
            cpu_intensity: 10,
            memory_usage: 10,
            network_usage: 10,
        },
    };

    let contract = Contract {
        code: vec![
            (OpCode::AllocateResource, Some(1)),
            (OpCode::CalculateVotingWeight, None),
            (OpCode::Return, None),
        ],
        state: HashMap::new(),
        required_reputation: 50,
        cooperative_metadata: metadata.clone(),
    };

    let mut vm = VM::new(100, reputation_system.scores.clone());
    let result = vm.execute_instruction(&OpCode::AllocateResource, Some(1), &metadata);
    assert!(result.is_ok(), "Failed to execute ALLOCATE_RESOURCE opcode");

    let result = vm.execute_instruction(&OpCode::CalculateVotingWeight, None, &metadata);
    assert!(result.is_ok(), "Failed to execute CALCULATE_VOTING_WEIGHT opcode");

    // Create governance proposals and vote
    let proposal1 = Proposal::new(1, ProposalType::Funding, String::from("Proposal for funding development."));
    proposal_history.add_proposal(proposal1);

    proposal_history.send_voting_reminder();
    proposal_history.proposals[0].vote(&sender_did.id, 10);
    proposal_history.proposals[0].close();

    assert_eq!(proposal_history.proposals.len(), 1, "Incorrect number of proposals in history");
    assert_eq!(proposal_history.proposals[0].total_votes(), 10, "Incorrect vote tally for proposal");
}
