-- Enable UUID extension for unique identifiers
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create blocks table
CREATE TABLE blocks (
    height BIGINT PRIMARY KEY,
    hash VARCHAR(64) UNIQUE NOT NULL,
    previous_hash VARCHAR(64) NOT NULL,
    timestamp BIGINT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on block hash
CREATE INDEX idx_blocks_hash ON blocks(hash);

-- Create transactions table
CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    hash VARCHAR(64) UNIQUE NOT NULL,
    block_height BIGINT REFERENCES blocks(height),
    sender VARCHAR(255) NOT NULL,
    transaction_type VARCHAR(50) NOT NULL,
    data JSONB NOT NULL,
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_transactions_hash ON transactions(hash);
CREATE INDEX idx_transactions_sender ON transactions(sender);
CREATE INDEX idx_transactions_block_height ON transactions(block_height);
CREATE INDEX idx_transactions_timestamp ON transactions(timestamp);

-- Create state table for key-value storage
CREATE TABLE state (
    key VARCHAR(255) PRIMARY KEY,
    value BYTEA NOT NULL,
    version BIGINT NOT NULL,
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on state key
CREATE INDEX idx_state_key ON state(key);

-- Create relationships table
CREATE TABLE relationships (
    id BIGSERIAL PRIMARY KEY,
    source_did VARCHAR(255) NOT NULL,
    target_did VARCHAR(255) NOT NULL,
    relationship_type VARCHAR(50) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_did, target_did, relationship_type)
);

CREATE INDEX idx_relationships_source ON relationships(source_did);
CREATE INDEX idx_relationships_target ON relationships(target_did);

-- Create reputation table
CREATE TABLE reputation (
    did VARCHAR(255) PRIMARY KEY,
    score INTEGER NOT NULL DEFAULT 0,
    context VARCHAR(50) NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    history JSONB DEFAULT '[]'::jsonb
);

CREATE INDEX idx_reputation_score ON reputation(score);
CREATE INDEX idx_reputation_context ON reputation(context);
