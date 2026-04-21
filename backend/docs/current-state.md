# Current State

Last updated: 2026-04-20

## Double-Entry Rewrite Progress

| Wave | Status | Branch |
|------|--------|--------|
| 1 — Account Hierarchy | Merged to main | interface-rework |
| 2 — Entries & Postings | Merged to main | wave-2-postings |
| 3 — Reports & Baselines | Complete | wave-3-reports |
| 4 — API Routes (rewrite) | Not started | — |
| 5 — Frontend Updates | Not started | — |

## What Wave 3 Did

- Added 3 new balance methods to Account: `balance_as_of_with_descendants`, `balance_between`, `balance_between_with_descendants`
- Created `domain/report.rs`: BalanceSheet (as_of date) and IncomeStatement (date range) with per-account detail + totals
- Created migration `00000000000013_baselines`: baselines table with account_id, period_type, period_start, amount
- Created `domain/baseline.rs`: BaselineRef, PeriodType (Monthly), Baseline with variance computation, NewBaseline builder
- Sign convention handled correctly: L/E negated in balance sheet equation, revenue negated for income statement display
- 132 tests total (113 unit + 19 integration), all pass

## What Wave 2 Did

- Created migration `00000000000011_postings`: postings table with entry_id, account_id, amount (integer cents), note, timestamps + indexes
- Created migration `00000000000012_rename_transactions_to_entries`: renamed transactions table → entries, transaction_id → entry_id
- Rewrote `domain/entry.rs`: PostingRef, Posting struct, PostingInput builder, NewEntry with zero-sum validation (min 2 postings, sum=0, no zero amounts), atomic insert
- Entry query methods on owning types: `AccountRef::entries()`, `Account::entries_with_descendants()`, `WorkspaceRef::entries()`
- Added balance methods to Account: `balance()`, `balance_with_descendants()`, `balance_as_of()`
- Updated `Account::soft_delete()` to block deletion when account has postings on active entries
- Balance route now returns real computed balance (was stub returning 0)
- Deleted dead `routes/transactions.rs` (old pre-double-entry code, was not declared in routes.rs)

## What Wave 1 Did

- Created migration `00000000000010_double_entry`: dropped categories, line_items, baselines tables; recreated accounts (hierarchical with parent_id) and transactions (slim)
- Rewrote `domain/account.rs`: AccountType enum, hierarchical account tree, NewAccount builder (root/child), create_default_tree(), traversal methods
- Deleted: `domain/category.rs`, `domain/baseline.rs`, `routes/categories.rs`, `routes/baselines.rs`, `routes/import.rs`
- Updated account routes for hierarchical creation (parent_id-based)
- 18 default accounts created on workspace creation

## What's Compiling and Tested

- `cargo test` — 19 integration tests + 113 unit tests pass (132 total)
- `cargo build` — clean (1 dead_code warning: `WorkspaceMember.user`)
- Schema, domain models, auth, permissions, routes all consistent

## Next: Wave 4 — API Routes

Rewrite all API routes to use the new domain model:
1. Entry CRUD routes (create, list, delete entries with postings)
2. Report routes (balance sheet, income statement)
3. Baseline CRUD routes (create, list, delete baselines, variance endpoint)
