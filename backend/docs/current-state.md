# Current State

Last updated: 2026-05-18

## Double-Entry Rewrite Progress

| Wave | Status | Branch |
|------|--------|--------|
| 1 — Account Hierarchy | Merged to main | interface-rework |
| 2 — Entries & Postings | Merged to main | wave-2-postings |
| 3 — Reports & Baselines | Merged to main | wave-3-reports |
| 4 — API Routes | Merged to main | wave-4-routes |
| 6 — Opening Balances + CSV Import | Complete | wave-6-import |
| 7 — Recurring Rules MVP | Merged to main | recurring-mvp |
| 7.5 — Recurring Hybrid Materialization | In progress | recurring-frontend |

## What Wave 7.5 Did

- Migration `00000000000019_entry_source_rule`: nullable `source_rule_id` FK on `entries` + partial index.
- `Entry` + `NewEntry` gained `source_rule` (Option<RecurringRuleRef>). `NewEntry::source_rule(rule)` is a chainable opt-in.
- `RecurringRule::materialize_due(up_to, conn)` walks the schedule, posts each occurrence via `NewEntry`, advances `last_fired_on`. Idempotent.
- `NewRecurringRule::insert` now calls `materialize_due(today)` inside the same transaction — past dtstart dumps backfill immediately, future dtstart stays dormant.
- `POST /workspaces/{ws}/recurring/runs` endpoint creates a run that iterates all active rules with `up_to = today`. Returns `{ posted: count, at: timestamp }`. Intended trigger: frontend calls on workspace load.
- `entry_to_json` now exposes `source_rule` in API responses.

## What Wave 7 Did

- Added `rrule` 0.14 crate dependency for RFC 5545 schedule parsing
- Migration `00000000000018_recurring`: `recurring_rules` + `recurring_posting_templates` tables (soft-delete via `deleted_at`, `last_fired_on` cursor for materialization)
- `domain/recurring.rs`: strong-typed wrappers — `Schedule` (validated RRULE + dtstart, exclusive `next_after`), `PostingTemplates` (≥2 / non-zero / zero-sum invariant), `RecurringRule` + `NewRecurringRule` builder, `WorkspaceRef::recurring_rules()`
- `routes/recurring.rs`: POST/GET list/GET one/DELETE under `/workspaces/{ws}/recurring`. Response embeds templates and computed `next_pending`
- Background daemon (cron) for materialization deliberately deferred — Wave 7.5 ships on-demand sync via endpoint instead

## What Wave 6 Did

- Added Opening Balances equity sub-account to the default account tree (created on workspace creation)

## What Wave 4 Did

- Created `routes/entries.rs`: POST (create with postings), GET list, GET single, DELETE (soft-delete)
- Created `routes/reports.rs`: GET balance-sheet (query param `as_of`), GET income-statement (query params `start_date`, `end_date`)
- Created `routes/baselines.rs`: POST create, GET list, DELETE (hard-delete), GET variance
- Added `Serialize` derives to domain types: Entry, Posting, BalanceSheet, IncomeStatement, AccountBalance, Baseline
- Added `Baseline::delete()` for hard-delete
- 16 new integration tests (35 total), 148 tests total, all pass

## What Wave 3 Did

- Added 3 new balance methods to Account: `balance_as_of_with_descendants`, `balance_between`, `balance_between_with_descendants`
- Created `domain/report.rs`: BalanceSheet (as_of date) and IncomeStatement (date range) with per-account detail + totals
- Created migration `00000000000013_baselines`: baselines table with account_id, period_type, period_start, amount
- Created `domain/baseline.rs`: BaselineRef, PeriodType (Monthly), Baseline with variance computation, NewBaseline builder
- Sign convention handled correctly: L/E negated in balance sheet equation, revenue negated for income statement display

## What Wave 2 Did

- Created migration `00000000000011_postings`: postings table with entry_id, account_id, amount (integer cents), note, timestamps + indexes
- Created migration `00000000000012_rename_transactions_to_entries`: renamed transactions table → entries, transaction_id → entry_id
- Rewrote `domain/entry.rs`: PostingRef, Posting struct, PostingInput builder, NewEntry with zero-sum validation (min 2 postings, sum=0, no zero amounts), atomic insert
- Entry query methods on owning types: `AccountRef::entries()`, `Account::entries_with_descendants()`, `WorkspaceRef::entries()`
- Added balance methods to Account: `balance()`, `balance_with_descendants()`, `balance_as_of()`
- Updated `Account::soft_delete()` to block deletion when account has postings on active entries
- Balance route now returns real computed balance (was stub returning 0)

## What Wave 1 Did

- Created migration `00000000000010_double_entry`: dropped categories, line_items, baselines tables; recreated accounts (hierarchical with parent_id) and transactions (slim)
- Rewrote `domain/account.rs`: AccountType enum, hierarchical account tree, NewAccount builder (root/child), create_default_tree(), traversal methods
- Deleted: `domain/category.rs`, `domain/baseline.rs`, `routes/categories.rs`, `routes/baselines.rs`, `routes/import.rs`
- Updated account routes for hierarchical creation (parent_id-based)
- 18 default accounts created on workspace creation

## What's Compiling and Tested

- `cargo test` — 60 integration tests + 198 unit tests pass (recurring MVP adds 16 unit + 7 integration)
- `cargo build` — clean (1 dead_code warning: `WorkspaceMember.user`)
- Schema, domain models, auth, permissions, routes all consistent
