
-- Add optimized indexes for common queries

-- Index for resources by type and owner
CREATE INDEX IF NOT EXISTS idx_resources_type_owner ON resources (resource_type, owner);

-- Index for proposals by status and federation
CREATE INDEX IF NOT EXISTS idx_proposals_status_federation ON proposals (status, federation_id);

-- Index for votes by proposal
CREATE INDEX IF NOT EXISTS idx_votes_proposal ON votes (proposal_id);

-- Index for federation members by status
CREATE INDEX IF NOT EXISTS idx_federation_members_status ON federation_members (federation_id, status);

-- Composite index for reputation scores by context
CREATE INDEX IF NOT EXISTS idx_reputation_context_did ON reputation_scores (context, did);

-- Full-text search index for proposal descriptions
CREATE INDEX IF NOT EXISTS idx_proposals_description_fts ON proposals USING gin(to_tsvector('english', description));

-- Time-range index for events
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (created_at);

-- Explain index usage statistics table
CREATE TABLE IF NOT EXISTS index_usage_stats (
    query_id TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    execution_plan TEXT NOT NULL,
    index_used BOOLEAN NOT NULL,
    execution_time_ms FLOAT NOT NULL,
    rows_processed INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add a function to log EXPLAIN output
CREATE OR REPLACE FUNCTION log_explain_output(
    query_id TEXT,
    query_text TEXT,
    execution_plan TEXT,
    index_used BOOLEAN,
    execution_time_ms FLOAT,
    rows_processed INTEGER
) RETURNS VOID AS $$
BEGIN
    INSERT INTO index_usage_stats (
        query_id, query_text, execution_plan, index_used, execution_time_ms, rows_processed
    ) VALUES (
        query_id, query_text, execution_plan, index_used, execution_time_ms, rows_processed
    ) ON CONFLICT (query_id) DO UPDATE SET
        execution_plan = EXCLUDED.execution_plan,
        index_used = EXCLUDED.index_used,
        execution_time_ms = EXCLUDED.execution_time_ms,
        rows_processed = EXCLUDED.rows_processed,
        timestamp = NOW();
END;
$$ LANGUAGE plpgsql;
