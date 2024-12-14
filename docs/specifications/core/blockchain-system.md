---
authors:
- Matt Faherty
date: '2024-11-03'
status: draft
title: Blockchain System
type: specification
version: 1.0.0
---

# Blockchain System

## Overview
The Blockchain System is the immutable ledger of the ICN, tracking all transactions, proposals, and reputation changes. Each cooperative or member transaction is recorded in the blockchain for transparency and traceability.

### Purpose
- **Transaction Integrity**: Ensures secure, tamper-resistant records of cooperative actions.
- **Reputation and Proposal Tracking**: Logs all reputation changes and governance activities.
- **Decentralized Ledger**: Provides a distributed record for cooperative actions and economic exchanges.

## Data Structures

### Block
- **index**: `u64` - Sequential block index.
- **previous_hash**: `String` - Hash of the previous block, linking it to the chain.
- **timestamp**: `u128` - Millisecond timestamp of creation.
- **transactions**: `Vec<Transaction>` - List of transactions included in the block.

### Transaction
- **sender**: `String` - DID of sender.
- **receiver**: `String` - DID of receiver.
- **amount**: `u64` - Value or resource exchanged.
- **hash**: `String` - Unique hash of the transaction, based on contents.

## Methods

### Add Transaction
Adds a new transaction to the pending list, verifying its contents before committing.

### Finalize Block
Bundles pending transactions into a block, calculating hash and adding to the blockchain.

### Calculate Hash
Computes the hash for each block, securing the data and linking blocks sequentially.

## Implementation Guidelines
- **Block Size Limit**: Define a maximum number of transactions per block to manage processing time.
- **Difficulty and Verification**: For scalability, adjust verification complexity based on load.

## Monitoring and Metrics
- **Transaction Throughput**: Measure number of transactions per block.
- **Hash Verification**: Track hash generation time to monitor performance.

## Modular Structure

The blockchain system modules are now split into smaller submodules for better separation of concerns. Below is the updated structure:

### blockchain/block.rs
- **block_creation**: Handles the creation of new blocks.
- **block_validation**: Manages the validation of blocks before adding to the chain.

### blockchain/chain.rs
- **chain_management**: Provides methods for managing the blockchain, including adding new blocks and retrieving the chain.

### blockchain/transaction.rs
- **transaction_creation**: Handles the creation of new transactions.
- **transaction_validation**: Manages the validation of transactions before adding to a block.
