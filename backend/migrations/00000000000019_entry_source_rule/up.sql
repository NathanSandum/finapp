ALTER TABLE entries ADD COLUMN source_rule_id TEXT REFERENCES recurring_rules(id);

CREATE INDEX idx_entries_source_rule ON entries(source_rule_id) WHERE source_rule_id IS NOT NULL;
