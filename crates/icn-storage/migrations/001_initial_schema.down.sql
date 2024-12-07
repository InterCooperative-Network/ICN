-- migrations/001_initial_schema.down.sql
DROP TRIGGER IF EXISTS update_key_value_updated_at ON key_value;
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP TABLE IF EXISTS key_value;