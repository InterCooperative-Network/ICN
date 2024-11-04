---
authors:
- Matt Faherty
date: '2024-11-03'
status: draft
title: Governance System
type: specification
version: 1.0.0
---

# Governance System

## Overview
The Governance System enables cooperative members to create, vote on, and track proposals within ICN. Proposals coordinate resource allocations, policy changes, and initiatives through reputation-weighted voting.

### Purpose
- **Decentralized Decision-Making**: Empowers cooperatives with direct influence over decisions.
- **Transparent Proposal Tracking**: Manages proposals throughout their lifecycle.
- **Reputation-Based Voting**: Aligns voting power with user contributions and engagement.

## Data Structures

### Proposal
- **id**: `u64` - Unique proposal identifier.
- **type**: `ProposalType` - Enum indicating proposal type (e.g., Funding).
- **status**: `ProposalStatus` - Enum indicating status (e.g., Open, Closed).
- **votes**: `Vec<(String, i64)>` - List of votes, with voter DID and weighted reputation.

### ProposalHistory
- **proposals**: `VecDeque<Proposal>` - Queue of proposals.
- **notifications**: `VecDeque<String>` - Queue of notifications for proposal events.

## Methods

### Create Proposal
Creates a proposal, setting parameters (type, description, duration) and initializing with Open status.

### Vote on Proposal
Registers a vote, using weighted reputation for transparency.

### Close Proposal
Locks in the proposal’s results, preventing further votes.

### Send Voting Reminder
Sends notifications to prompt voting for proposals near closure.

## Implementation Guidelines
- **Reputation Requirements**: Define minimum reputation for creating and voting to ensure credibility.
- **Standard Proposal Types**: Classify proposal types (Funding, PolicyChange) for consistency.

## Monitoring and Metrics
- **Proposal Activity**: Track proposal creation and voting rates.
- **Outcome Recording**: Log vote distribution and final outcome for cooperative accountability.
