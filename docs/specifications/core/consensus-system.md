---
authors:
  - Matt Faherty
date: '2024-11-18'
status: draft
title: Consensus System Specification
type: specification
version: 1.0.0
---

# Consensus System Documentation

## Overview

The Consensus System is a fundamental component of the Inter-Cooperative Network (ICN). It ensures agreement across the network for proposals, resource sharing, and decision-making. The consensus mechanism used by ICN is designed to be resilient, scalable, and align with cooperative values, fostering a democratic and inclusive governance model.

### Purpose
- **Collective Decision-Making**: Provide a robust process for cooperatives to reach collective agreements.
- **Trustless Validation**: Enable nodes to reach consensus without relying on centralized authority, maintaining decentralization.
- **Accountability and Transparency**: Ensure that all consensus decisions are auditable and traceable to maintain the integrity of the network.

## 1. Consensus Types

### 1.1 Proof of Cooperation (PoC)
Proof of Cooperation is the core consensus mechanism used within ICN. It leverages a cooperative model where validators, called `cooperators`, work together to validate transactions and reach consensus.
- **Use Case**: General-purpose agreement for governance, resource allocation, and proposal approvals.
- **Validator Selection**: Validators are chosen based on reputation scores and cooperative involvement.

### 1.2 Federated Consensus
Federated consensus is used for specific federations within ICN that require more localized decision-making. This mechanism allows federated groups to reach agreement independently of the broader network.
- **Use Case**: Federations making autonomous decisions about local resource allocation or governance.

## 2. System Components

### 2.1 Consensus Nodes
Consensus nodes, known as `cooperators`, are responsible for participating in consensus rounds. Each cooperative in the network can nominate one or more nodes to act as cooperators.

- **Validator Eligibility**: Nodes must meet minimum reputation and contribution criteria to be eligible as cooperators.
- **Validator Roles**: Validators propose, validate, and finalize transactions and proposals.

### 2.2 Consensus Round Lifecycle
The consensus process is divided into distinct rounds:
- **Proposal Stage**: A proposal or transaction is initiated by a member and broadcast to cooperators.
- **Validation Stage**: Cooperators validate the proposed changes to ensure compliance with cooperative standards and policies.
- **Voting Stage**: Cooperators vote on the proposal. A threshold must be reached for consensus.
- **Finalization Stage**: Upon successful voting, the proposal is finalized and committed to the network.

### 2.3 Quorum and Thresholds
- **Quorum Requirement**: A minimum number of cooperators must participate in each round to proceed.
- **Approval Threshold**: The number of affirmative votes required to achieve consensus can vary based on the proposal type (e.g., 50%, 66%, or 75%).

## 3. Key Methods

### 3.1 Initiating a Consensus Round
Any cooperative member can initiate a consensus round by creating a proposal. The proposal is then broadcast to all validators for review.

#### Initiate Consensus
```rust
pub struct ConsensusProposal {
    pub proposal_id: String,
    pub proposer: String,
    pub description: String,
    pub created_at: u64,
    pub status: ConsensusStatus,
}
```
- **Input**: `proposal_id` (unique identifier), `proposer` (DID of proposer), `description` (proposal details).
- **Functionality**: Creates and broadcasts a proposal to cooperators for review.

### 3.2 Voting on a Proposal
Validators participate in voting to determine whether a proposal should be accepted or rejected.

#### Cast Validator Vote
```rust
pub fn cast_validator_vote(&mut self, proposal_id: &str, validator: &str, approve: bool) -> Result<(), String> {
    if !self.is_eligible(validator) {
        return Err("Validator not eligible".to_string());
    }
    let vote = Vote {
        validator: validator.to_string(),
        approve,
        timestamp: current_timestamp(),
    };
    if let Some(proposal) = self.proposals.get_mut(proposal_id) {
        proposal.votes.push(vote);
        Ok(())
    } else {
        Err("Proposal not found".to_string())
    }
}
```
- **Input**: `proposal_id` (ID of the proposal), `validator` (DID of the validator), `approve` (approval or rejection).
- **Functionality**: Records a vote for the proposal.

### 3.3 Finalizing Consensus
After reaching the required threshold, the proposal can be finalized.

#### Finalize Proposal
```rust
pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<(), String> {
    if let Some(proposal) = self.proposals.get_mut(proposal_id) {
        if self.has_reached_threshold(proposal) {
            proposal.status = ConsensusStatus::Finalized;
            Ok(())
        } else {
            Err("Threshold not reached".to_string())
        }
    } else {
        Err("Proposal not found".to_string())
    }
}
```
- **Input**: `proposal_id` (ID of the proposal).
- **Functionality**: Marks the proposal as finalized if consensus is achieved.

## 4. Security Considerations

### 4.1 Sybil Attack Prevention
- **Reputation-Based Eligibility**: Only nodes with sufficient reputation can act as validators, reducing the risk of Sybil attacks where malicious actors create multiple identities to influence consensus.
- **Contribution Verification**: Nodes must prove ongoing contributions to maintain their validator status.

### 4.2 Byzantine Fault Tolerance
The consensus mechanism is designed to be resilient against Byzantine faults, with redundancy and multiple cooperators ensuring that no single point of failure can compromise the system.

## 5. Implementation Guidelines

### 5.1 Performance Requirements
- **Efficient Validator Selection**: Use indexed data structures for rapid selection of eligible validators.
- **Scalability**: Ensure that the consensus mechanism can accommodate an increasing number of validators without a drop in performance.

### 5.2 Testing Requirements
- **Unit Testing**: Include tests for consensus initiation, validator voting, and proposal finalization methods.
- **Network Simulation**: Simulate different network conditions, including validator failures, to ensure that the system behaves as expected.

## 6. Future Considerations

### 6.1 Dynamic Validator Pools
Implement dynamic adjustments to the validator pool size based on network conditions to ensure optimal performance and resilience.

### 6.2 Cross-Federation Consensus
Develop mechanisms for enabling federations to coordinate on large-scale decisions involving multiple independent federations, facilitating shared initiatives.

## Appendix

### A. Summary of Consensus Methods
- **Initiate Consensus**: Begins a new consensus round for a proposal.
- **Cast Validator Vote**: Allows a validator to vote on a proposal.
- **Finalize Proposal**: Marks a proposal as finalized once it meets the approval threshold.

### B. Modular Structure

The consensus system modules are now split into smaller submodules for better separation of concerns. Below is the updated structure:

#### consensus/proof_of_cooperation/mod.rs
- **round_management**: Handles the management of consensus rounds.
- **validation**: Manages the validation of proposals and transactions.
- **timeout_handling**: Provides methods for handling consensus timeouts and error logging.

#### consensus/round.rs
- **round_initialization**: Handles the initialization of new consensus rounds.
- **round_finalization**: Manages the finalization of consensus rounds.

#### consensus/validator.rs
- **validator_selection**: Provides methods for selecting validators based on reputation and contribution.
- **validator_roles**: Manages the roles and responsibilities of validators.
