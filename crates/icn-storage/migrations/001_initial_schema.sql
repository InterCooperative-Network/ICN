-- migrations/001_initial_schema.sql
CREATE TABLE IF NOT EXISTS key_value (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS key_value_key_idx ON key_value(key);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_key_value_updated_at
    BEFORE UPDATE ON key_value
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();