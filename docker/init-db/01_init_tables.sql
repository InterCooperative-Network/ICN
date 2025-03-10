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

-- Create proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    created_by TEXT NOT NULL,
    ends_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    did TEXT NOT NULL
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id BIGSERIAL PRIMARY KEY,
    proposal_id BIGINT REFERENCES proposals(id),
    voter TEXT NOT NULL,
    approve BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(proposal_id, voter)
);

-- Create identities table
CREATE TABLE IF NOT EXISTS identities (
    did VARCHAR PRIMARY KEY,
    public_key TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create reputation_scores table
CREATE TABLE IF NOT EXISTS reputation_scores (
    did VARCHAR REFERENCES identities(did),
    category VARCHAR(50) NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (did, category)
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

-- Create storage table for key-value storage
CREATE TABLE IF NOT EXISTS storage (
    key TEXT PRIMARY KEY,
    value BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indices for performance
CREATE INDEX IF NOT EXISTS idx_validator_reputation ON validators(reputation_score DESC);
CREATE INDEX IF NOT EXISTS idx_reputation_scores_did ON reputation_scores(did);
CREATE INDEX IF NOT EXISTS idx_votes_proposal ON votes(proposal_id);
CREATE INDEX IF NOT EXISTS idx_federation_members_did ON federation_members(member_did);
CREATE INDEX IF NOT EXISTS idx_storage_updated ON storage(updated_at);