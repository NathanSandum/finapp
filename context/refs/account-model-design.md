# Account Model — Canonical vs Custom

Design discussion for separating canonical (system) accounts from user-created (custom) accounts so the 3-statement model has stable, type-checked anchors.

## Motivation

The double-entry engine is solid, but the 3-statement model (Income Statement / Balance Sheet / Cash Flow) and personal-finance ratios depend on identifying specific accounts (cash, emergency fund, retained earnings, etc.) that don't fall out of the type system. Today the frontend matches them by name string (`"Chequing"`, casing-folded), which is fragile in two directions:

- A user renaming `Chequing` to `Main Checking` silently breaks every statement and ratio that depended on it.
- Statement code can refer to a name that doesn't exist in a given workspace and silently fall back to zero.

The trigger that surfaced this: the Overview's **Emergency Fund** ratio currently computes `liquidAssets / 3moExpenseAvg`, where `liquidAssets` is the sum of accounts whose name matches a `CASH_NAMES` set. The user's mental model is different — Emergency Fund should be a *specific designated account* whose balance is divided by typical monthly expense. The fix isn't local; it's a systemic gap. We don't have a stable way to say "this account is the Emergency Fund."

## Current State

- One `accounts` table. Every account has an `account_type` (`asset | liability | equity | revenue | expense`) on its root, derived for descendants via `parent_id`.
- Default tree seeded on workspace creation (`backend/src/domain/account.rs:537`): the five roots, plus `Assets:{Chequing, Savings, TFSA, RRSP}`, `Liabilities:Credit Card`, `Revenue:Employment`, `Expenses:{Food, Rent, Groceries, Dining, Transit, Utilities, Entertainment}`, `Equity:Opening Balances`.
- Frontend statement code uses `topBucket()` to roll a leaf's balance up to its top-level child of a root (the "bucket"), then matches the bucket's name to identify cash. See `frontend/src/lib/views/overview/compute.ts` (`CASH_NAMES`, `isCash`).
- No type-system distinction between accounts the engine *requires* and accounts the user *chose to add*. Everything is one flat `Account` type.

## Proposed Model

Split accounts into two conceptual classes — **canonical** (a.k.a. system) and **custom** — with the distinction enforced as deeply as practical, ideally at the Rust type level so a 3SM function literally cannot be called with a wrong account.

### Schema

Add `system_role: Option<SystemRole>` to `accounts`:

- `Some(role)` — canonical row. Seeded on workspace creation. Role is immutable. Row is undeletable. Display name editable (so user can rename `Chequing` → `Main Checking`) but `system_role` is the stable identifier.
- `None` — custom account. Created by the user. Must descend from a canonical via `parent_id`. Inherits the canonical's behavior in 3SM rollups via the existing `top_bucket` traversal.

Uniqueness: at most one canonical row per `(workspace_id, system_role)`.

### Rust Type System

```rust
pub enum SystemRole {
    Cash,
    EmergencyFund,
    RetainedEarnings,
    OpeningBalances,
    // ...open question, see below
}

pub struct Account { /* existing fields, plus system_role: Option<SystemRole> */ }

/// Newtype guaranteeing system_role.is_some(). Constructed only via
/// `Account::into_system()` or `lookup_system(workspace, role)`.
pub struct SystemAccount(Account);

impl SystemAccount {
    pub fn role(&self) -> SystemRole { self.0.system_role.unwrap() }
    pub fn account(&self) -> &Account { &self.0 }
}
```

3SM and ratio functions take `SystemAccount`, never raw `Account`. Calling `compute_emergency_fund` with a non-canonical account is a compile error. Custom accounts that descend from a canonical are pulled in implicitly via the existing parent_id walk — the canonical is the contract surface, descendants are an implementation detail of the rollup.

### Frontend

Mirrors the backend shape: `Account` carries `system_role: SystemRole | null`. Statement compute code looks up the canonical by `system_role`, never by name. The Accounts tab (see Scope below) renders the hierarchy with canonical roots visually distinct (badge, locked rename-only, no-delete).

## Open Questions

1. **Canonical role set.** What's the minimum viable list for the 3SM and the personal-finance ratios on Overview today?
   - Strong yes: `Cash` (cash anchor used everywhere), `RetainedEarnings` (Balance Sheet equity rollup), `OpeningBalances` (initial balance entries).
   - Probably yes: `EmergencyFund` (Overview ratio), since the user's mental model treats it as a designated account.
   - Maybe: distinct anchors for `Investments`, `Debt`, `Income`, `Expenses` umbrella roots — or do those stay as plain `account_type` roots without a role?
   - Decide before locking the enum: every addition is a migration + code change.

2. **Cash: one or many.** Is `Cash` a single canonical account (the user's primary chequing) with all other liquid accounts as custom siblings or descendants? Or can multiple accounts hold the `Cash` role (then "cash" in 3SM = sum of all cash-roled accounts)?
   - Single is simpler and matches the existing "Chequing-only" anchor (per memory `cash_anchor_chequing_only.md`).
   - Multiple is more flexible (e.g. someone with two chequing accounts) but every statement query becomes a sum-over-set instead of a lookup.

3. **Tree vs DAG.** Today accounts form a strict tree via `parent_id`. Does any envisioned use case need a custom account to belong to multiple canonicals? Default to keeping it a tree unless we can name a concrete need.

4. **Display name editability on canonical rows.** Confirm: user can rename `Chequing` → `Main Checking` (display label), but `system_role` stays `Cash`. Statements continue to work because they query by role, not name.

5. **Migration of existing data.** Existing workspaces have seeded accounts that pre-date roles. Migration must:
   - Backfill `system_role` on the matching seeded names (e.g. find the `Assets:Chequing` row, set `system_role = Cash`).
   - Decide what happens if a user already renamed `Chequing` away from the default name — best-effort matching on the old default plus a per-workspace prompt? Or hard fail and require manual assignment?

6. **Accounts tab scope.** Suggested initial scope:
   - Tree view of all accounts grouped by root type, canonical rows badged.
   - Create custom child under any account.
   - Rename any account (canonical: display name only).
   - Delete custom accounts (canonical: blocked).
   - Show balance per account.
   - Out of scope for v1: reparenting accounts, merging accounts, archiving.

## Migration Considerations

- New column on `accounts`: `system_role TEXT NULL` with a CHECK constraint or app-side validation against the enum strings.
- Unique index on `(workspace_id, system_role) WHERE system_role IS NOT NULL` (assuming single-cardinality decision in Q2).
- Backfill migration runs once per existing workspace; decision in Q5 governs the strategy.
- Frontend cache shape changes (new field on Account); existing tabs are unaffected as long as they keep working from `account_type`. New Overview/3SM lookups switch to role-based.

## Out of Scope (Initial)

- The Accounts tab itself — design the model first, build the tab as a follow-up PR.
- Refactoring statement compute to use roles — separate PR after the model lands.
- Removing the `CASH_NAMES` string-match fallback — keep the fallback until the role is plumbed end-to-end.
- Accrual/projection features that may later need additional canonical roles (e.g. `AccountsPayable` for accrued interest). Add roles as concrete need arises, never speculatively.

## Handoff Pointer

Next agent: read this file plus `backend/src/domain/account.rs` (current Account model) and `frontend/src/lib/views/overview/compute.ts` (current string-match anchors). Resolve the open questions with the user before writing schema/code. The `cavekit-ledger.md` kit will need a new requirement (`R10: System Account Roles` or similar) once decisions are locked.
