# On-Chain vs. Off-Chain Data

## Introduction
This document clarifies which data is canonical (must be on-chain) and which data is ephemeral/convenience-only (stored in Postgres).

## Canonical Data
- **On-Chain Data**: List the types of data that must be stored on-chain to maintain trust and integrity.
  - Example: Transaction records, governance proposals.

## Ephemeral Data
- **Off-Chain Data**: List the types of data that can be stored off-chain in Postgres.
  - Example: User preferences, temporary cache data.

## Data Flow
- **Data Reconstruction**: Describe the process for reconstructing off-chain data from the blockchain if needed.
- **Synchronization**: Outline how data is synchronized between the blockchain and Postgres.

## Security Considerations
- **Data Integrity**: Ensure that off-chain data can be verified against on-chain records to prevent tampering.
