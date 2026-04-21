CREATE TABLE baselines (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    account_id   TEXT NOT NULL REFERENCES accounts(id),
    period_type  TEXT NOT NULL DEFAULT 'monthly',
    period_start TEXT NOT NULL,
    amount       INTEGER NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, account_id, period_start)
);
