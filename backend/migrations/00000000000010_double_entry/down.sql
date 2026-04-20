-- Reverse of double-entry migration (best-effort, destructive)

DROP TABLE IF EXISTS accounts;

CREATE TABLE accounts (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name         TEXT NOT NULL,
    account_type TEXT NOT NULL,
    currency     TEXT NOT NULL DEFAULT 'CAD',
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, name)
);

DROP TABLE IF EXISTS transactions;

CREATE TABLE transactions (
    id              TEXT PRIMARY KEY NOT NULL,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    account_id      TEXT NOT NULL REFERENCES accounts(id),
    transfer_account_id TEXT REFERENCES accounts(id),
    date            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    amount          INTEGER NOT NULL,
    cleared         BOOLEAN NOT NULL DEFAULT 0,
    deleted_at      TIMESTAMP,
    created_at      TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at      TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE categories (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name         TEXT NOT NULL,
    description  TEXT,
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, name)
);

CREATE TABLE line_items (
    id              TEXT PRIMARY KEY NOT NULL,
    transaction_id  TEXT NOT NULL REFERENCES transactions(id),
    category_id     TEXT NOT NULL REFERENCES categories(id),
    amount          INTEGER NOT NULL,
    note            TEXT,
    created_at      TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at      TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE baselines (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    category_id  TEXT NOT NULL REFERENCES categories(id),
    period_type  TEXT NOT NULL,
    period_start TEXT NOT NULL,
    amount       INTEGER NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, category_id, period_start)
);
