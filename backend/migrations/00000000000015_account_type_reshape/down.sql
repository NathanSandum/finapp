PRAGMA foreign_keys = OFF;

DROP INDEX IF EXISTS accounts_custom_name;
DROP INDEX IF EXISTS accounts_one_per_system;

CREATE TABLE accounts_new (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    parent_id    TEXT REFERENCES accounts_new(id),
    name         TEXT NOT NULL,
    account_type TEXT NOT NULL,
    currency     TEXT NOT NULL DEFAULT 'CAD',
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, parent_id, name)
);

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;

PRAGMA foreign_keys = ON;
