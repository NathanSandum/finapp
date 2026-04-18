CREATE TABLE baselines (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    category_id  TEXT NOT NULL REFERENCES categories(id),
    period_type  TEXT NOT NULL, -- monthly, quarterly
    period_start TEXT NOT NULL, -- YYYY-MM-DD (first day of the period)
    amount       INTEGER NOT NULL, -- cents
    created_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    updated_at   TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, category_id, period_start)
);
