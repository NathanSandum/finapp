CREATE TABLE recurring_rules (
    id             TEXT PRIMARY KEY NOT NULL,
    workspace_id   TEXT NOT NULL REFERENCES workspaces(id),
    description    TEXT NOT NULL,
    rrule          TEXT NOT NULL,
    dtstart        TEXT NOT NULL,
    last_fired_on  TEXT,
    deleted_at     TIMESTAMP,
    created_at     TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at     TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_recurring_rules_workspace ON recurring_rules(workspace_id) WHERE deleted_at IS NULL;

CREATE TABLE recurring_posting_templates (
    id              TEXT PRIMARY KEY NOT NULL,
    rule_id         TEXT NOT NULL REFERENCES recurring_rules(id),
    account_id      TEXT NOT NULL REFERENCES accounts(id),
    amount          INTEGER NOT NULL,
    note            TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at      TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_recurring_posting_templates_rule ON recurring_posting_templates(rule_id);
