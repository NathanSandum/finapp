-- Add role column to accounts for typed anchor accounts.
-- One account per (workspace, role) when role is non-NULL.

ALTER TABLE accounts ADD COLUMN role TEXT;

-- Backfill: root accounts get *_root, plus cash + opening_equity.
UPDATE accounts SET role = 'assets_root'
    WHERE parent_id IS NULL AND account_type = 'asset' AND deleted_at IS NULL;
UPDATE accounts SET role = 'liabilities_root'
    WHERE parent_id IS NULL AND account_type = 'liability' AND deleted_at IS NULL;
UPDATE accounts SET role = 'equity_root'
    WHERE parent_id IS NULL AND account_type = 'equity' AND deleted_at IS NULL;
UPDATE accounts SET role = 'revenue_root'
    WHERE parent_id IS NULL AND account_type = 'revenue' AND deleted_at IS NULL;
UPDATE accounts SET role = 'expenses_root'
    WHERE parent_id IS NULL AND account_type = 'expense' AND deleted_at IS NULL;
UPDATE accounts SET role = 'cash'
    WHERE name = 'Chequing' AND account_type = 'asset' AND parent_id IS NOT NULL AND deleted_at IS NULL;
UPDATE accounts SET role = 'opening_equity'
    WHERE name = 'Opening Balances' AND account_type = 'equity' AND parent_id IS NOT NULL AND deleted_at IS NULL;

CREATE UNIQUE INDEX idx_accounts_role
    ON accounts(workspace_id, role)
    WHERE role IS NOT NULL AND deleted_at IS NULL;
