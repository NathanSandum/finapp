CREATE TABLE accounts (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name         TEXT NOT NULL,
    account_type TEXT NOT NULL, -- chequing, savings, tfsa, rrsp, brokerage, credit, other
    currency     TEXT NOT NULL DEFAULT 'CAD',
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, name)
);
