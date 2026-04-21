# Current State

Last updated: 2026-04-20

## Double-Entry Rewrite Progress

| Wave | Status | Branch |
|------|--------|--------|
| 1 — Account Hierarchy | Merged to main | interface-rework |
| 2 — Entries & Postings | Merged to main | wave-2-postings |
| 3 — Reports & Baselines | Not started | — |
| 4 — API Routes (rewrite) | Not started | — |
| 5 — Frontend Updates | Not started | — |

## What Wave 2 Did

- Created migration `00000000000011_postings`: postings table with entry_id, account_id, amount (integer cents), note, timestamps + indexes
- Created migration `00000000000012_rename_transactions_to_entries`: renamed transactions table → entries, transaction_id → entry_id
- Rewrote `domain/entry.rs`: PostingRef, Posting struct, PostingInput builder, NewEntry with zero-sum validation (min 2 postings, sum=0, no zero amounts), atomic insert
- Entry query methods on owning types: `AccountRef::entries()`, `Account::entries_with_descendants()`, `WorkspaceRef::entries()`
- Added balance methods to Account: `balance()`, `balance_with_descendants()`, `balance_as_of()`
- Updated `Account::soft_delete()` to block deletion when account has postings on active entries
- Balance route now returns real computed balance (was stub returning 0)
- Deleted dead `routes/transactions.rs` (old pre-double-entry code, was not declared in routes.rs)
- 107 tests total (88 unit + 19 integration), all pass

## What Wave 1 Did

- Created migration `00000000000010_double_entry`: dropped categories, line_items, baselines tables; recreated accounts (hierarchical with parent_id) and transactions (slim)
- Rewrote `domain/account.rs`: AccountType enum, hierarchical account tree, NewAccount builder (root/child), create_default_tree(), traversal methods
- Deleted: `domain/category.rs`, `domain/baseline.rs`, `routes/categories.rs`, `routes/baselines.rs`, `routes/import.rs`
- Updated account routes for hierarchical creation (parent_id-based)
- 18 default accounts created on workspace creation

## What's Compiling and Tested

- `cargo test` — 19 integration tests + 88 unit tests pass (107 total)
- `cargo build` — clean (1 dead_code warning: `WorkspaceMember.user`)
- Schema, domain models, auth, permissions, routes all consistent

## Next: Wave 3 — Reports & Baselines

From `context/plans/build-double-entry.md`:

1. **T3.1** — Balance sheet computation (assets, liabilities, equity, net income)
2. **T3.2** — Income statement computation (revenue and expenses for period)
3. **T3.3** — Baselines table rewrite (point to accounts instead of categories), variance computation
