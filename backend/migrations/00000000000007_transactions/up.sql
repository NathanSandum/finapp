CREATE TABLE transactions (
    id              TEXT PRIMARY KEY NOT NULL,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    account_id      TEXT NOT NULL REFERENCES accounts(id),
    transfer_account_id TEXT REFERENCES accounts(id),
    date            TEXT NOT NULL, -- YYYY-MM-DD
    description     TEXT NOT NULL DEFAULT '',
    amount          INTEGER NOT NULL, -- cents
    cleared         BOOLEAN NOT NULL DEFAULT 0,
    deleted_at      TIMESTAMP,
    created_at      TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at      TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE line_items (
    id              TEXT PRIMARY KEY NOT NULL,
    transaction_id  TEXT NOT NULL REFERENCES transactions(id),
    category_id     TEXT NOT NULL REFERENCES categories(id),
    amount          INTEGER NOT NULL, -- cents, must be non-zero
    note            TEXT,
    created_at      TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at      TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);
