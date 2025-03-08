use std::sync::{Arc, Mutex};
use icn_types::{Block, Transaction, FederationOperation};
use crate::identity::IdentitySystem;
use crate::blockchain::Blockchain;
use crate::governance::{ProposalHistory, Proposal, ProposalType, handle_federation_operation};
use crate::reputation::ReputationSystem;

pub struct BlockchainService {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl BlockchainService {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self { blockchain }
    }
    
    pub async fn add_transaction(&self, transaction: Transaction) -> Result<(), String> {
        let mut chain = self.blockchain.lock().unwrap();
        chain.add_transaction(transaction).await
    }
    
    pub async fn add_block(&self, block: Block) -> Result<(), String> {
        let mut chain = self.blockchain.lock().unwrap();
        chain.add_block(block).await
    }
    
    pub fn get_latest_block(&self) -> Option<Block> {
        let chain = self.blockchain.lock().unwrap();
        chain.blocks.last().cloned()
    }
}

pub struct IdentityService {
    identity_system: Arc<Mutex<IdentitySystem>>,
}

impl IdentityService {
    pub fn new(identity_system: Arc<Mutex<IdentitySystem>>) -> Self {
        Self { identity_system }
    }
    
    pub fn register_identity(&self, did: crate::identity::DID, permissions: Vec<String>) -> Result<(), String> {
        let mut system = self.identity_system.lock().unwrap();
        system.register_did(did, permissions);
        Ok(())
    }
    
    pub fn has_permission(&self, did: &str, permission: &str) -> bool {
        let system = self.identity_system.lock().unwrap();
        system.has_permission(did, permission)
    }
}

pub struct GovernanceService {
    proposal_history: Arc<Mutex<ProposalHistory>>,
}

impl GovernanceService {
    pub fn new(proposal_history: Arc<Mutex<ProposalHistory>>) -> Self {
        Self { proposal_history }
    }
    
    pub fn add_proposal(&self, proposer: String, proposal_type: ProposalType) -> Result<Proposal, String> {
        let mut history = self.proposal_history.lock().unwrap();
        let proposal = Proposal::new(proposer, proposal_type);
        let proposal_clone = proposal.clone();
        history.add_proposal(proposal);
        Ok(proposal_clone)
    }
    
    pub fn vote(&self, voter: String, _proposal_id: String, approve: bool) -> Result<(), String> {
        let mut history = self.proposal_history.lock().unwrap();
        history.vote(voter, _proposal_id, approve)
    }
    
    pub fn get_proposal(&self, id: String) -> Option<Proposal> {
        let history = self.proposal_history.lock().unwrap();
        history.get_proposal(id)
    }
    
    pub fn execute_proposal(&self, proposal_id: &str) -> Result<(), String> {
        let mut history = self.proposal_history.lock().unwrap();
        history.execute_proposal(proposal_id)
    }
    
    pub async fn handle_federation_operation(&self, operation: FederationOperation) -> Result<String, String> {
        handle_federation_operation(operation).await
    }
}
