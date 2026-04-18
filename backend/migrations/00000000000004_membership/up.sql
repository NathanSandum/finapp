CREATE TABLE membership_events (
    id           TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id      TEXT NOT NULL REFERENCES users(id),
    actor_id     TEXT NOT NULL REFERENCES users(id),
    event_type   TEXT NOT NULL, -- owner_assigned | invited | invitation_accepted | role_changed | removed | left
    role         TEXT,          -- owner | editor | viewer (null for removed/left)
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id      TEXT NOT NULL REFERENCES users(id),
    role         TEXT NOT NULL, -- owner | editor | viewer
    joined_at    TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (workspace_id, user_id)
);
