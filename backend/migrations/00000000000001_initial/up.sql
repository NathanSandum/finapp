-- Initial migration: placeholder tables
-- Domain-specific columns are added in subsequent migrations per wave.

CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS workspaces (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
