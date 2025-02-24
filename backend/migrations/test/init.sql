-- Test database initialization script
CREATE TABLE IF NOT EXISTS proposals (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    created_by VARCHAR(255) NOT NULL,
    ends_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS votes (
    proposal_id INTEGER REFERENCES proposals(id),
    voter VARCHAR(255) NOT NULL,
    approve BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (proposal_id, voter)
);

CREATE TABLE IF NOT EXISTS identities (
    did VARCHAR(255) PRIMARY KEY,
    public_key TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS reputation_scores (
    did VARCHAR(255) REFERENCES identities(did),
    category VARCHAR(50) NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (did, category)
);

CREATE TABLE IF NOT EXISTS federations (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    federation_type VARCHAR(50) NOT NULL,
    terms JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS federation_members (
    federation_id VARCHAR(255) REFERENCES federations(id),
    member_did VARCHAR(255) REFERENCES identities(did),
    role VARCHAR(50) NOT NULL,
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    PRIMARY KEY (federation_id, member_did)
);

CREATE TABLE IF NOT EXISTS resources (
    id VARCHAR(255) PRIMARY KEY,
    resource_type VARCHAR(50) NOT NULL,
    owner VARCHAR(255) REFERENCES identities(did),
    total_amount BIGINT NOT NULL,
    available_amount BIGINT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS resource_allocations (
    id VARCHAR(255) PRIMARY KEY,
    resource_id VARCHAR(255) REFERENCES resources(id),
    requester VARCHAR(255) REFERENCES identities(did),
    amount BIGINT NOT NULL,
    allocated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'active'
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_proposals_created_by ON proposals(created_by);
CREATE INDEX IF NOT EXISTS idx_reputation_scores_did ON reputation_scores(did);
CREATE INDEX IF NOT EXISTS idx_federation_members_member_did ON federation_members(member_did);
CREATE INDEX IF NOT EXISTS idx_resources_owner ON resources(owner);
CREATE INDEX IF NOT EXISTS idx_resource_allocations_requester ON resource_allocations(requester); 