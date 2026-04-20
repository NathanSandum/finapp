-- Wave 1: Double-entry bookkeeping — hierarchical accounts, drop categories/line_items/baselines

-- Drop tables that depend on categories and old transaction shape
DROP TABLE IF EXISTS line_items;
DROP TABLE IF EXISTS baselines;

-- Recreate transactions without account_id, transfer_account_id, amount
-- (postings replace these in Wave 2)
CREATE TABLE transactions_new (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    date         TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    cleared      BOOLEAN NOT NULL DEFAULT 0,
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

-- Recreate accounts with parent_id for hierarchy, account_type as root-derived enum
CREATE TABLE accounts_new (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    parent_id    TEXT REFERENCES accounts_new(id),
    name         TEXT NOT NULL,
    account_type TEXT NOT NULL, -- asset, liability, equity, revenue, expense
    currency     TEXT NOT NULL DEFAULT 'CAD',
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, parent_id, name)
);

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;

-- Drop categories — replaced by hierarchical accounts
DROP TABLE IF EXISTS categories;
