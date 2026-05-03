# Build Site — Double-Entry Bookkeeping Rewrite

Stack: Axum, Diesel, SQLite, Tauri, SvelteKit.
Workflow: TDD (failing test first, then implementation), PR per wave.

This build site covers the rewrite from single-entry (transactions + line items + categories) to double-entry (transactions + postings + hierarchical accounts). Foundation, auth, and workspace layers are unchanged — only ledger, baselines, and their API routes are affected.

---

## Wave 1 — Account Hierarchy

*No dependencies on other waves. Replaces the flat account model and eliminates categories entirely.*

### T1.1 — Schema Migration: Hierarchical Accounts
**Kit:** cavekit-ledger.md R1
**Tasks:**
- Write new migration that:
  - Adds `parent_id TEXT REFERENCES accounts(id)` to `accounts` table
  - Removes `account_type` column from `accounts` (type is now derived from root ancestor)
  - Drops `categories` table
  - Drops `line_items` table
  - Drops `baselines` table (will be recreated in Wave 3 referencing accounts)
  - Updates `transactions` table: drops `account_id`, `transfer_account_id`, `amount` columns (postings replace these)
- Update Diesel `schema.rs` to reflect new table shapes
- Verify migration runs against fresh in-memory DB

### T1.2 — Account Domain Model with Hierarchy
**Kit:** cavekit-ledger.md R1
**Tasks:**
- Write failing tests:
  - Create root account succeeds (parent_id = NULL)
  - Create child account under root succeeds
  - Create deeply nested account succeeds (3+ levels)
  - Two children of same parent cannot share name
  - Children of different parents can share name
  - `full_name()` returns colon-delimited path (e.g. `Assets:Bank:Chequing`)
  - `account_type()` returns type derived from root ancestor
  - `descendants()` returns all transitive children
  - `ancestors()` returns path from root to self
- Rewrite `Account` struct: remove `account_type` field, add `parent_id: Option<AccountRef>`
- Implement `full_name()`, `account_type()`, `descendants()`, `ancestors()`
- Make tests pass

### T1.3 — Default Account Tree
**Kit:** cavekit-ledger.md R1
**Tasks:**
- Write failing tests:
  - Workspace creation produces five root accounts: Assets, Liabilities, Equity, Revenue, Expenses
  - Workspace creation produces default sub-accounts under each root (e.g. `Assets:Chequing`, `Assets:Savings`, `Expenses:Food`, `Expenses:Rent`, `Revenue:Employment`)
  - Root accounts cannot be deleted
- Replace `Category::create_defaults()` with `Account::create_default_tree()` in workspace creation flow
- Remove entire `domain/category.rs` module
- Make tests pass

---

## Wave 2 — Transactions & Postings

*Depends on Wave 1. Replaces single-entry transactions with double-entry.*

### T2.1 — Posting Model & Zero-Sum Invariant
**Kit:** cavekit-ledger.md R2, R3
**Tasks:**
- Write new migration: `postings` table (`id TEXT PRIMARY KEY`, `transaction_id TEXT NOT NULL REFERENCES transactions(id)`, `account_id TEXT NOT NULL REFERENCES accounts(id)`, `amount INTEGER NOT NULL`, `note TEXT`, `created_at TIMESTAMP`, `updated_at TIMESTAMP`)
- Write failing tests:
  - Create transaction with two postings that sum to zero succeeds
  - Create transaction with three+ postings that sum to zero succeeds
  - Create transaction where postings do not sum to zero is rejected
  - Create transaction with fewer than two postings is rejected
  - Posting amount of zero is rejected
  - Adding a posting that breaks zero-sum is rejected
  - Updating a posting that breaks zero-sum is rejected
  - Deleting a posting that would leave fewer than two is rejected
- Implement `Posting` struct with `id`, `transaction_id`, `account_id`, `amount`, `note`, timestamps
- Rewrite `Transaction` struct: remove `account_id`, `transfer_account_id`, `amount`; keep `id`, `workspace_id`, `date`, `description`, `cleared`, `deleted_at`, timestamps
- Rewrite `NewTransaction` builder: accept list of `PostingInput` (account + amount + optional note), validate zero-sum and minimum two postings, insert atomically
- Make tests pass

### T2.2 — Balance Computation
**Kit:** cavekit-ledger.md R4
**Tasks:**
- Write failing tests:
  - Account balance = sum of cleared postings for that account
  - Account balance includes descendant account postings
  - Balance as-of date filters correctly
  - Pending postings excluded from cleared balance
  - Fundamental equation holds: Assets = Liabilities + Equity + (Revenue - Expenses)
  - Soft-deleted transactions excluded from balance
- Implement `Account::balance()` and `Account::balance_as_of()` using postings + descendant rollup
- Make tests pass

### T2.3 — Transaction Queries
**Kit:** cavekit-ledger.md R7
**Tasks:**
- Write failing tests:
  - Filter by account returns transactions with any posting to that account or its descendants
  - Filter by date range works (inclusive)
  - Filter by cleared status works
  - Pagination works (default 50, max 200)
  - Results ordered by date descending
  - Soft-deleted excluded
- Rewrite `Transaction` query methods to filter via postings table joins
- Make tests pass

### T2.4 — Soft Deletes
**Kit:** cavekit-ledger.md R8
**Tasks:**
- Write failing tests:
  - Soft-deleting account with active descendants is rejected
  - Soft-deleting account with all-deleted descendants succeeds
  - Soft-deleted account excluded from listings and balances
  - Soft-deleted transaction excluded from queries and balances
  - Soft-deleted records recoverable
- Update soft-delete logic for new hierarchy constraints
- Make tests pass

---

## Wave 3 — Reports & Baselines

*Depends on Wave 2.*

### T3.1 — Balance Sheet
**Kit:** cavekit-ledger.md R5
**Tasks:**
- Write failing tests:
  - Balance sheet includes Assets, Liabilities, Equity accounts with balances
  - Net income (Revenue - Expenses) shown as Equity component
  - Balance sheet balances: assets = liabilities + equity + net income
  - Balance sheet respects as-of date
  - Accounts with zero balance omitted (or included — decide and test)
- Implement balance sheet computation: query account tree, roll up balances, compute net income
- Make tests pass

### T3.2 — Income Statement
**Kit:** cavekit-ledger.md R6
**Tasks:**
- Write failing tests:
  - Income statement includes Revenue and Expenses accounts with period activity
  - Net income = total revenue - total expenses
  - Only activity within date range included
  - Accounts with zero activity omitted
- Implement income statement computation
- Make tests pass

### T3.3 — Baselines (Repointed to Accounts)
**Kit:** cavekit-baselines.md R1, R2, R3
**Tasks:**
- Write new migration: `baselines` table (`id`, `workspace_id`, `account_id TEXT NOT NULL REFERENCES accounts(id)`, `period_type`, `period_start`, `amount`, `created_at`, `updated_at`) with unique index on `(workspace_id, account_id, period_start)`
- Write failing tests:
  - Create baseline for account succeeds
  - Duplicate account+period fails
  - Variance = actual postings sum - baseline for account and period
  - Account with no baseline returns null variance
  - Period summary includes accounts with activity or baseline
  - Summary filterable by account (including descendants)
- Rewrite `Baseline` struct: `category_id` → `account_id`
- Rewrite variance computation to aggregate postings instead of line items
- Make tests pass

---

## Wave 4 — API Routes

*Depends on Wave 2, Wave 3.*

### T4.1 — Account Endpoints (Rewrite)
**Kit:** cavekit-api.md R5
**Tasks:**
- Write failing integration tests:
  - `POST /workspaces/:id/accounts` creates account with optional `parent_id`
  - `GET /workspaces/:id/accounts` returns hierarchical tree with balances
  - `GET /workspaces/:id/accounts/:accountId` returns account with rolled-up balance
  - `PATCH /workspaces/:id/accounts/:accountId` renames account
  - `DELETE /workspaces/:id/accounts/:accountId` fails if active descendants exist
- Rewrite route handlers: remove `account_type` from create, add `parent_id`; tree serialization for list
- Remove `routes/categories.rs` entirely
- Make tests pass

### T4.2 — Transaction Endpoints (Rewrite)
**Kit:** cavekit-api.md R6
**Tasks:**
- Write failing integration tests:
  - `POST /workspaces/:id/transactions` accepts postings array, enforces zero-sum
  - `GET /workspaces/:id/transactions` filters by account (including descendants)
  - Individual posting CRUD endpoints enforce invariants
- Rewrite route handlers: replace line_items with postings in request/response shapes
- Make tests pass

### T4.3 — Report Endpoints (New)
**Kit:** cavekit-api.md R7
**Tasks:**
- Write failing integration tests:
  - `GET /workspaces/:id/reports/balance-sheet?as_of=DATE` returns correct structure
  - `GET /workspaces/:id/reports/income-statement?start=DATE&end=DATE` returns correct structure
  - Both return hierarchical account trees with rolled-up balances
- Implement report route handlers
- Make tests pass

### T4.4 — Baseline Endpoints (Rewrite)
**Kit:** cavekit-api.md R8
**Tasks:**
- Write failing integration tests:
  - Upsert references account_id instead of category_id
  - Summary returns variance per account
- Rewrite baseline route handlers
- Make tests pass

### T4.5 — Import Endpoint (Rewrite)
**Kit:** —
**Tasks:**
- Update bulk import endpoint to create double-entry transactions (each imported row becomes a transaction with two postings: one to source account, one to expense/income account)
- Update CSV mapping logic
- Write failing tests, then make pass

