CREATE TABLE postings (
    id             TEXT PRIMARY KEY NOT NULL,
    transaction_id TEXT NOT NULL REFERENCES transactions(id),
    account_id     TEXT NOT NULL REFERENCES accounts(id),
    amount         INTEGER NOT NULL,
    note           TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at     TIMESTAMP NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_postings_transaction ON postings(transaction_id);
CREATE INDEX idx_postings_account ON postings(account_id);
