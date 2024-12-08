-- Initial schema migration for ICN storage system
-- This creates the core tables needed for the system

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Blocks table for storing blockchain data
CREATE TABLE blocks (
    height BIGINT PRIMARY KEY,
    hash VARCHAR(64) UNIQUE NOT NULL,
    previous_hash VARCHAR(64) NOT NULL,
    timestamp BIGINT NOT NULL,
    data JSONB NOT NULL,
    merkle_root VARCHAR(64) NOT NULL,
    validator_signature VARCHAR(128) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_hash CHECK (length(hash) = 64),
    CONSTRAINT valid_previous_hash CHECK (length(previous_hash) = 64)
);

CREATE INDEX idx_blocks_hash ON blocks(hash);
CREATE INDEX idx_blocks_timestamp ON blocks(timestamp);

-- Transactions table for individual operations
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    hash VARCHAR(64) UNIQUE NOT NULL,
    block_height BIGINT REFERENCES blocks(height),
    sender VARCHAR(255) NOT NULL,
    recipient VARCHAR(255),
    transaction_type VARCHAR(50) NOT NULL,
    amount NUMERIC(20, 8),
    data JSONB NOT NULL,
    timestamp BIGINT NOT NULL,
    signature VARCHAR(128) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_tx_hash CHECK (length(hash) = 64)
);

CREATE INDEX idx_transactions_hash ON transactions(hash);
CREATE INDEX idx_transactions_block_height ON transactions(block_height);
CREATE INDEX idx_transactions_sender ON transactions(sender);
CREATE INDEX idx_transactions_recipient ON transactions(recipient);
CREATE INDEX idx_transactions_timestamp ON transactions(timestamp);

-- State table for current network state
CREATE TABLE state (
    key VARCHAR(255) PRIMARY KEY,
    value BYTEA NOT NULL,
    version BIGINT NOT NULL,
    last_modified TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_state_version ON state(version);

-- Metrics table for system monitoring
CREATE TABLE metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_type VARCHAR(50) NOT NULL,
    value NUMERIC NOT NULL,
    tags JSONB DEFAULT '{}',
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_metrics_type_timestamp ON metrics(metric_type, timestamp);

-- Create metrics partitioning
CREATE TABLE metrics_recent PARTITION OF metrics
    FOR VALUES FROM (CURRENT_TIMESTAMP - INTERVAL '24 hours') 
    TO (CURRENT_TIMESTAMP + INTERVAL '24 hours');

CREATE TABLE metrics_historical PARTITION OF metrics
    DEFAULT;

-- Triggers for automatic partition management
CREATE OR REPLACE FUNCTION create_metrics_partition()
RETURNS trigger AS $$
BEGIN
    CREATE TABLE IF NOT EXISTS metrics_p_
        PARTITION OF metrics
        FOR VALUES FROM (date_trunc('day', CURRENT_TIMESTAMP))
        TO (date_trunc('day', CURRENT_TIMESTAMP + INTERVAL '1 day'));
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_metrics_partition_trigger
    AFTER INSERT ON metrics
    EXECUTE FUNCTION create_metrics_partition();