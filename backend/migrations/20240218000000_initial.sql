CREATE TABLE IF NOT EXISTS proposals (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    created_by TEXT NOT NULL,
    ends_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS votes (
    id BIGSERIAL PRIMARY KEY,
    proposal_id BIGINT NOT NULL REFERENCES proposals(id),
    voter TEXT NOT NULL,
    approve BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(proposal_id, voter)
);

CREATE INDEX idx_resource_owner ON resources(owner);
CREATE INDEX idx_resource_type ON resources(resource_type);
CREATE INDEX idx_resource_created_at ON resources(created_at);
CREATE INDEX idx_resource_updated_at ON resources(updated_at);
