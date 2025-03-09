-- Create extensions and schemas
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create application schemas
CREATE SCHEMA IF NOT EXISTS icn_core;
CREATE SCHEMA IF NOT EXISTS icn_identity;
CREATE SCHEMA IF NOT EXISTS icn_federation;

-- Create schema for the ICN application
CREATE SCHEMA IF NOT EXISTS icn;

-- Identity table to store DIDs and related information
CREATE TABLE IF NOT EXISTS icn.identities (
    did VARCHAR(255) PRIMARY KEY,
    public_key TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Proposals table for governance
CREATE TABLE IF NOT EXISTS icn.proposals (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    created_by VARCHAR(255) NOT NULL REFERENCES icn.identities(did),
    ends_at TIMESTAMP NOT NULL,
    did VARCHAR(255) NOT NULL REFERENCES icn.identities(did),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Votes table for proposal voting
CREATE TABLE IF NOT EXISTS icn.votes (
    proposal_id INTEGER REFERENCES icn.proposals(id),
    voter VARCHAR(255) REFERENCES icn.identities(did),
    approve BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (proposal_id, voter)
);

-- Reputation scoring system
CREATE TABLE IF NOT EXISTS icn.reputation_scores (
    did VARCHAR(255) REFERENCES icn.identities(did),
    category VARCHAR(50) NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (did, category)
);

-- Federation management
CREATE TABLE IF NOT EXISTS icn.federations (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    federation_type VARCHAR(50) NOT NULL,
    terms JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'active'
);

-- Federation membership
CREATE TABLE IF NOT EXISTS icn.federation_members (
    federation_id VARCHAR(255) REFERENCES icn.federations(id),
    member_did VARCHAR(255) REFERENCES icn.identities(did),
    role VARCHAR(50) NOT NULL,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    PRIMARY KEY (federation_id, member_did)
);

-- Resource management
CREATE TABLE IF NOT EXISTS icn.resources (
    id VARCHAR(255) PRIMARY KEY,
    resource_type VARCHAR(50) NOT NULL,
    owner VARCHAR(255) REFERENCES icn.identities(did),
    total_amount BIGINT NOT NULL,
    available_amount BIGINT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Resource allocation tracking
CREATE TABLE IF NOT EXISTS icn.resource_allocations (
    id VARCHAR(255) PRIMARY KEY,
    resource_id VARCHAR(255) REFERENCES icn.resources(id),
    requester VARCHAR(255) REFERENCES icn.identities(did),
    amount BIGINT NOT NULL,
    allocated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'active'
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_proposals_created_by ON icn.proposals(created_by);
CREATE INDEX IF NOT EXISTS idx_reputation_scores_did ON icn.reputation_scores(did);
CREATE INDEX IF NOT EXISTS idx_federation_members_member_did ON icn.federation_members(member_did);
CREATE INDEX IF NOT EXISTS idx_resources_owner ON icn.resources(owner);
CREATE INDEX IF NOT EXISTS idx_resource_allocations_requester ON icn.resource_allocations(requester);

-- Create a health check function
CREATE OR REPLACE FUNCTION icn.health_check()
RETURNS TEXT AS $$
BEGIN
    RETURN 'ok';
END;
$$ LANGUAGE plpgsql;

-- Set up permissions
GRANT USAGE ON SCHEMA icn TO icnuser;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA icn TO icnuser;
GRANT EXECUTE ON FUNCTION icn.health_check TO icnuser;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA icn TO icnuser;