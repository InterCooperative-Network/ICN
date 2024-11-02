// tests/integration_test.rs
use icn_backend::{
    blockchain::{Blockchain, transaction::Transaction},
    identity::DID,
    reputation::ReputationSystem,
    governance::{Proposal, ProposalType, ProposalHistory},
    vm::{VM, Contract, CooperativeMetadata, OpCode, ResourceImpact},
};