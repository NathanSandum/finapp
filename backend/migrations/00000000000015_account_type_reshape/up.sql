-- Reshape `account_type` from category enum (asset/liability/...) to System/Custom
-- discriminator. Category becomes a derived property on SystemAccount variants.
-- `name` becomes nullable (System rows have no name).

PRAGMA foreign_keys = OFF;

CREATE TABLE accounts_new (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    parent_id    TEXT REFERENCES accounts_new(id),
    name         TEXT,
    account_type TEXT NOT NULL CHECK (account_type IN (
        'AssetsRoot', 'LiabilitiesRoot', 'EquityRoot', 'RevenueRoot', 'ExpensesRoot',
        'Chequing', 'Savings', 'RetainedEarnings', 'OpeningBalances',
        'Custom'
    )),
    currency     TEXT NOT NULL DEFAULT 'CAD',
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    CHECK ((account_type = 'Custom') = (name IS NOT NULL AND parent_id IS NOT NULL))
);

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;

CREATE UNIQUE INDEX accounts_one_per_system
    ON accounts(workspace_id, account_type) WHERE account_type != 'Custom';

CREATE UNIQUE INDEX accounts_custom_name
    ON accounts(workspace_id, parent_id, name) WHERE account_type = 'Custom';

PRAGMA foreign_keys = ON;
