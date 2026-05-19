DROP INDEX IF EXISTS idx_entries_source_rule;
ALTER TABLE entries DROP COLUMN source_rule_id;
