---
authors:
  - Matt Faherty
date: '2024-11-18'
status: draft
title: Voting System Specification
type: specification
version: 1.0.0
---

# Voting System Documentation

## Overview

The Voting System is an essential component of the Inter-Cooperative Network (ICN). It provides the mechanisms for decision-making within cooperatives and federations, allowing all members to participate in governance processes. The system supports a variety of voting types and includes measures to ensure fairness, transparency, and alignment with cooperative values.

### Purpose
- **Democratic Decision-Making**: Ensure all members have an opportunity to participate in governance through a fair and transparent voting process.
- **Weighted Influence**: Allow certain decisions to be influenced by participant contributions, while maintaining core democratic equality in key votes.
- **Efficient Governance**: Facilitate the approval or rejection of proposals in a structured and systematic manner.

## 1. Voting Types

### 1.1 Simple Majority Voting
In simple majority voting, a proposal is approved if more than 50% of the votes cast are in favor.
- **Use Case**: Routine decisions that impact the cooperative but do not involve sensitive or irreversible changes.

### 1.2 Supermajority Voting
Supermajority voting requires a larger consensus, typically 66% or 75% of the votes.
- **Use Case**: Decisions that impact cooperative bylaws, membership changes, or resource allocation that affects other members significantly.

### 1.3 Weighted Voting
Weighted voting adjusts voting power based on reputation or contribution. While ensuring equality in core decisions, weighted voting helps reflect contributions in non-critical proposals.
- **Use Case**: Decisions involving economic contributions, cooperative expansion, or technical upgrades.

## 2. System Components

### 2.1 Proposal Lifecycle
A proposal passes through several phases from initiation to final decision.

- **Initiation**: A member creates a proposal, specifying the action, goals, and resource requirements.
- **Discussion**: Members debate and discuss the proposal, with revisions permitted before the voting stage.
- **Voting**: The voting window is set, and eligible members cast their votes.
- **Result**: Votes are counted and the result is announced, followed by any necessary implementation steps.

### 2.2 Proposal Types
- **Policy Proposals**: Proposals that affect governance policies, cooperative rules, or federated guidelines.
- **Resource Allocation Proposals**: Involve the distribution or commitment of cooperative resources.
- **Membership Proposals**: Include adding or removing members, modifying membership rights, or imposing sanctions.

### 2.3 Voting Requirements
Voting requirements depend on the type of proposal:
- **Quorum**: A minimum number of participants must cast their votes for the result to be considered valid.
- **Eligibility**: Participants must have a minimum reputation score to vote on certain proposals, ensuring informed decision-making.

## 3. Key Methods

### 3.1 Creating a Proposal
Any eligible member can create a proposal, detailing the objectives, expected outcomes, and resource requirements.

#### Create Proposal
```rust
pub struct Proposal {
    pub proposal_id: String,
    pub proposer: String,
    pub proposal_type: String,
    pub description: String,
    pub resources: Option<HashMap<String, u64>>,
    pub created_at: u64,
    pub status: ProposalStatus,
}
```
- **Input**: `proposal_id` (unique identifier), `proposer` (DID of proposer), `proposal_type` (type of proposal), `description` (details), `resources` (optional, resource requirements).
- **Functionality**: Initializes a new proposal with relevant details.

### 3.2 Voting on a Proposal
Eligible members can cast their votes for a proposal. Each proposal specifies the type of vote required (simple majority, supermajority, etc.).

#### Cast Vote
```rust
pub fn cast_vote(&mut self, proposal_id: &str, voter: &str, approve: bool) -> Result<(), String> {
    if !self.is_eligible(voter) {
        return Err("Voter not eligible".to_string());
    }
    let vote = Vote {
        voter: voter.to_string(),
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
- **Input**: `proposal_id` (ID of the proposal), `voter` (DID of the voter), `approve` (approval or rejection).
- **Functionality**: Records the vote, provided that the voter is eligible.

### 3.3 Counting Votes
After the voting period ends, the votes are counted to determine if the proposal is approved or rejected.

#### Count Votes
```rust
pub fn count_votes(&self, proposal_id: &str) -> Result<(u32, u32), String> {
    if let Some(proposal) = self.proposals.get(proposal_id) {
        let approvals = proposal.votes.iter().filter(|v| v.approve).count() as u32;
        let rejections = proposal.votes.len() as u32 - approvals;
        Ok((approvals, rejections))
    } else {
        Err("Proposal not found".to_string())
    }
}
```
- **Input**: `proposal_id` (ID of the proposal).
- **Output**: A tuple with the number of approvals and rejections.

## 4. Security Considerations

### 4.1 Vote Integrity
- **Identity Verification**: Votes can only be cast by verified members with DIDs, ensuring that only valid participants influence decision-making.
- **Immutable Voting Record**: Once cast, votes are recorded in an immutable format to prevent tampering or alteration.

### 4.2 Prevention of Voting Fraud
- **Minimum Reputation Requirement**: Only members who meet a minimum reputation threshold are eligible to vote, reducing the risk of vote manipulation.
- **Quorum Enforcement**: Proposals require a quorum to prevent a small number of participants from passing significant decisions without broader consensus.

## 5. Implementation Guidelines

### 5.1 Performance Requirements
- **Efficient Proposal Lookup**: Use hash maps for efficient access to proposals and voting data.
- **Scalability**: Ensure the system can handle multiple concurrent votes and proposals without a performance drop, even as the cooperative network grows.

### 5.2 Testing Requirements
- **Unit Testing**: Include unit tests for critical methods, such as `create_proposal`, `cast_vote`, and `count_votes`.
- **Simulated Scenarios**: Test the voting system under different scenarios, including edge cases like tied votes or missed quorums, to ensure robustness.

## 6. Future Considerations

### 6.1 Integration with Reputation
Integrate the reputation system so that participants with a higher reputation may have more influence in non-critical decisions, while maintaining an egalitarian approach for essential votes.

### 6.2 Delegated Voting
Implement a mechanism for delegated voting, where members can assign their voting rights to trusted representatives, ensuring that all voices are represented even if participants cannot vote directly.

## Appendix

### A. Summary of Voting Methods
- **Create Proposal**: Initializes a proposal for consideration.
- **Cast Vote**: Allows an eligible member to vote on a proposal.
- **Count Votes**: Tallies votes to determine the outcome of a proposal.

### B. Modular Structure

The voting management modules are now split into smaller submodules for better separation of concerns. Below is the updated structure:

#### voting/proposal.rs
- **creation**: Handles the creation of voting proposals.
- **submission**: Manages the submission and tracking of proposals.
- **voting**: Provides methods for casting votes and tallying results.
