-- Create the schema
CREATE SCHEMA IF NOT EXISTS icn;

-- Set the search path
SET search_path TO icn;

-- Create validators table
CREATE TABLE IF NOT EXISTS validators (
    did VARCHAR PRIMARY KEY,
    reputation_score BIGINT NOT NULL DEFAULT 50,
    last_block_proposed TIMESTAMP,
    last_active TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR NOT NULL DEFAULT 'active'
);

-- Create reputation_history table
CREATE TABLE IF NOT EXISTS reputation_history (
    id SERIAL PRIMARY KEY,
    did VARCHAR NOT NULL REFERENCES validators(did),
    change_amount BIGINT NOT NULL,
    reason VARCHAR NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create consensus_rounds table
CREATE TABLE IF NOT EXISTS consensus_rounds (
    round_id SERIAL PRIMARY KEY,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMP,
    coordinator_did VARCHAR REFERENCES validators(did),
    block_hash VARCHAR,
    status VARCHAR NOT NULL DEFAULT 'in_progress'
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id SERIAL PRIMARY KEY,
    round_id BIGINT REFERENCES consensus_rounds(round_id),
    validator_did VARCHAR REFERENCES validators(did),
    vote BOOLEAN NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(round_id, validator_did)
);

-- Create federations table
CREATE TABLE IF NOT EXISTS federations (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR NOT NULL DEFAULT 'active'
);

-- Create federation_members table
CREATE TABLE IF NOT EXISTS federation_members (
    federation_id VARCHAR REFERENCES federations(id),
    member_did VARCHAR REFERENCES validators(did),
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    role VARCHAR NOT NULL DEFAULT 'member',
    PRIMARY KEY (federation_id, member_did)
);

-- Create indices for performance
CREATE INDEX IF NOT EXISTS idx_validator_reputation ON validators(reputation_score DESC);
CREATE INDEX IF NOT EXISTS idx_reputation_history_did ON reputation_history(did);
CREATE INDEX IF NOT EXISTS idx_votes_round ON votes(round_id);
CREATE INDEX IF NOT EXISTS idx_federation_members_did ON federation_members(member_did);