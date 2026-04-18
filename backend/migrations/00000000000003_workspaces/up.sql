ALTER TABLE workspaces ADD COLUMN name TEXT NOT NULL DEFAULT '';
ALTER TABLE workspaces ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
