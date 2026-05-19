# Domain Model

## Entity Relationships

```
User ──< WorkspaceUser >── Workspace
              │                 │
              │                 ├──< Account (tree via parent_id)
              │                 │       └── AccountType: asset|liability|equity|revenue|expense
              │                 │
              │                 ├──< Entry ──< Posting >── Account
              │                 │             (amount in cents, zero-sum per entry)
              │                 │
              │                 └──< Baseline >── Account
              │                      (monthly budget per account, variance computed on demand)
         MembershipEvent (append-only log)
```

## Ref Pattern

Every entity has a `FooRef` newtype wrapping `String` (UUID v4 as text). Derived via `DieselNewType`. Used for type-safe foreign keys and path parameters.

- `UserRef`, `WorkspaceRef`, `AccountRef`, `EntryRef`, `PostingRef`, `BaselineRef`
- Each entity has `as_ref() -> &FooRef` and `from_ref(ref, conn) -> Result<Option<Foo>>`
- Connection parameter is always last

## Enums Stored as Text

`AccountType`, `Role`, and `PeriodType` all follow same pattern:
- `as_str() -> &'static str` / `from_str(s) -> Option<Self>`
- `FromSql<Text, Sqlite>` / `ToSql<Text, Sqlite>` impls
- Serde with `rename_all = "snake_case"` (AccountType, PeriodType) or manual (Role)

### AccountType
`Asset`, `Liability`, `Equity`, `Revenue`, `Expense`

### Role
`Viewer` < `Editor` < `Owner` — implements `Ord` for role comparison. `WorkspaceMember::require_role()` checks minimum role.

### PeriodType
`Monthly` — extensible for future period types.

## Account Hierarchy

Accounts form a tree rooted at five types. `parent_id` is nullable — NULL means root.

- `account_type` is denormalized on every row, inherited from root ancestor at insert time
- Uniqueness: `UNIQUE(workspace_id, parent_id, name)` in DB for siblings. Root uniqueness (NULL parent) enforced in application code (SQLite treats NULL as distinct)
- `NewAccount::child(parent, name)` — inherits workspace, account_type, currency from parent (only public constructor)
- `NewAccount::anchored_root(workspace, name, account_type)` — seed-only, tags with the matching root anchor role
- `NewAccount::anchored_child(parent, name, ChildAnchor)` — seed-only, tags with a non-root anchor role (Cash, OpeningEquity)
- `.currency(str)` builder method overrides default "CAD"
- Roots are sealed: the public API rejects `parent_id == null`. The five typed roots are created exactly once per workspace by `create_default_tree`.

## AccountRole

Anchor roles tag the well-known accounts the 3-statement model depends on. Stored as nullable TEXT; partial unique index `idx_accounts_role` enforces one active account per `(workspace_id, role)`.

Variants: `AssetsRoot`, `LiabilitiesRoot`, `EquityRoot`, `RevenueRoot`, `ExpensesRoot`, `Cash`, `OpeningEquity`.

Anchor accounts can be renamed freely (role identifies, not name). Anchor accounts cannot be soft-deleted (see Soft Deletes).

### Default Tree (created on workspace creation)
```
Assets: Chequing, Savings, TFSA, RRSP
Liabilities: Credit Card
Equity: Opening Balances
Revenue: Employment
Expenses: Food, Rent, Groceries, Dining, Transit, Utilities, Entertainment
```
Total: 5 roots + 14 children = 19 accounts

### Traversal Methods (all take `&mut SqliteConnection` last)
- `full_name()` — colon-delimited path: "Expenses:Food:Cafes"
- `ancestors()` — root-to-parent chain (excludes self)
- `children()` — direct children (active only)
- `descendants()` — BFS all transitive children (active only)

## Membership (Event-Sourced)

Two tables work together:
- `membership_events` — append-only log: invited, accepted, role_changed, removed, left, owner_assigned
- `workspace_members` — current-state projection, updated transactionally with each event

Permission checks read from projection. At-least-one-owner invariant enforced by checking projection before writing remove/leave event, within transaction.

## Auth Flow

1. Register/Login returns raw token (UUID v4)
2. Token stored as SHA-256 hash in `sessions` table
3. `AuthUser` extractor: reads Bearer token from header, validates via `Session::validate()`
4. `WorkspaceMember` extractor: extends `AuthUser`, extracts workspace ID from URI path, checks membership + role
5. Sessions expire after 1 week. Logout deletes session row.

## Error Handling

`AppError` enum maps to HTTP:
| Variant | Status | When |
|---------|--------|------|
| `NotFound(msg)` | 404 | Entity not found |
| `Forbidden` | 403 | Not a member or insufficient role |
| `Unauthorized` | 401 | Missing/invalid/expired token |
| `Conflict(msg)` | 409 | Uniqueness violation, last-owner removal |
| `Validation(fields)` | 422 | Field-level validation errors |
| `Internal(msg)` | 500 | Message NOT leaked to client |

`DieselError` auto-converts: `UniqueViolation` → Conflict, `NotFound` → NotFound, else → Internal.

## Entries & Postings (Double-Entry)

Every entry has 2+ postings that sum to zero. Amounts in integer cents (i32).

### Entry
- Fields: id, workspace, date (TEXT "YYYY-MM-DD"), description, deleted_at, timestamps, source_rule (Option<RecurringRuleRef> — set when generated from a recurring rule, null for manual entries)
- `postings(conn)` — all postings for this entry
- Query methods live on owning types:
  - `AccountRef::entries(conn)` — entries with posting to this account, excluding soft-deleted
  - `Account::entries_with_descendants(conn)` — same but includes descendant accounts
  - `WorkspaceRef::entries(conn)` — all workspace entries

### Posting
- Fields: id, entry (EntryRef), account (AccountRef), amount (i32 cents), note, timestamps
- No independent CRUD — created atomically with entry

### NewEntry Builder
- `NewEntry::new(workspace, date, description, postings: Vec<PostingInput>)`
- `insert(conn)` validates: min 2 postings, sum == 0, no zero amounts. Atomic insert.
- `PostingInput::new(account, amount)` with `.note(str)` chainable

### Balance Methods (on Account)
- `balance(conn) -> i64` — sum of postings for this account on non-deleted entries
- `balance_with_descendants(conn) -> i64` — includes all descendant account postings
- `balance_as_of(date, conn) -> i64` — filtered by entry date <= date
- `balance_as_of_with_descendants(date, conn) -> i64` — combines date filter + descendants
- `balance_between(start, end, conn) -> i64` — date range (inclusive both ends)
- `balance_between_with_descendants(start, end, conn) -> i64` — date range + descendants
- Return i64 (not i32) to handle overflow from summing many postings

### Sign Convention
Positive postings = debits, negative = credits. Revenue/liability/equity accounts have negative normal balance. Sum of all account balances across entire workspace = 0.

## Reports

### BalanceSheet
- `BalanceSheet::as_of(workspace, date, conn)` — produces balance sheet as of given date
- Contains per-account detail (children of each root) + totals for assets, liabilities, equity
- `net_income = -(revenue_total + expense_total)` — revenue is negative in ledger, expenses positive
- `total_liabilities_and_equity = -total_liabilities - total_equity + net_income` — L and E negated since they're credit-normal in ledger
- Accounting equation: `total_assets == total_liabilities_and_equity`

### IncomeStatement
- `IncomeStatement::for_period(workspace, start, end, conn)` — revenue and expenses for date range
- `total_revenue` = negated from ledger (displayed as positive)
- `total_expenses` = raw from ledger (already positive for debit-normal expenses)
- `net_income = total_revenue - total_expenses`
- Per-account detail uses `balance_between_with_descendants` for descendant rollup

### AccountBalance
Shared struct for report detail: `{ account: AccountRef, name: String, balance: i64 }`

## Baselines

Monthly budget targets per account.

### Baseline
- Fields: id, workspace, account, period_type (Monthly), period_start (YYYY-MM-DD), amount (i32 cents), timestamps
- `variance(conn) -> i64` — computes actual spending (`balance_between_with_descendants` for the month) minus baseline amount. Positive = over budget.
- Uses `end_of_month(period_start)` helper for date computation

### NewBaseline Builder
- `NewBaseline::monthly(workspace, account, period_start, amount)` constructor
- `insert(conn)` — unique on (workspace, account, period_start)

### Query Methods
- `AccountRef::baselines(conn)` — baselines for this account
- `WorkspaceRef::baselines(conn)` — all baselines in workspace

## Recurring Rules

Workspace-scoped templates that describe a repeating entry. CRUD only — materialization (auto-posting on schedule) is deferred. See `context/kits/cavekit-recurring.md`.

### Strong types

- `Schedule` — newtype around `(rrule: String, dtstart: NaiveDate)`. The only constructor `Schedule::parse(rrule, dtstart)` parses and validates the RRULE against `dtstart` (via the `rrule` crate). Existence of a `Schedule` value is proof of well-formedness. Method `next_after(date_exclusive) -> Option<NaiveDate>` is strictly-after — internally bumps the inclusive `RRuleSet::after` by one day.
- `PostingTemplates` — newtype around `Vec<PostingTemplateInput>`. Constructor enforces ≥2 entries, non-zero amounts, and zero-sum. `into_inner()` drops the wrapper to consume the inner vec.

### RecurringRule
- Fields: id, workspace, description, rrule, dtstart, last_fired_on (Option<NaiveDate>), deleted_at, timestamps
- `from_ref(ref, conn)` — only returns active (non-deleted) rules
- `templates(conn)` — owning posting templates
- `soft_delete(conn)` — sets deleted_at; excluded from default queries
- `schedule()` — reconstructs `Schedule` from stored cols (Internal error if storage corrupt)
- `next_occurrence_after(date, conn)` — delegates to `schedule().next_after(date)`
- `next_pending()` — uses `last_fired_on` as cursor, falls back to `dtstart - 1 day` so the first call returns `dtstart`

### NewRecurringRule
- `NewRecurringRule::new(workspace, description, schedule: Schedule, templates: PostingTemplates)` — accepts only validated newtypes, so the constructor itself cannot produce an invalid rule
- `insert(conn)` — atomic insert of rule row + template rows + **backfill** in a single transaction. After writing the rule and its templates, calls `materialize_due(today, conn)` so any past occurrences are posted as real entries immediately. Future-dated rules stay dormant until the materialize endpoint fires.

### Materialization (Hybrid Pattern)
Past occurrences become real `entries` rows (with `source_rule_id = rule.id`). Future occurrences remain virtual — they're computed from the rule on demand via `Schedule::next_after`.

- `RecurringRule::materialize_due(up_to, conn) -> Vec<Entry>` — walks the schedule from `next_pending` cursor through `up_to`, posts each occurrence via `NewEntry::insert(...).source_rule(rule.id)`, and advances `last_fired_on` to the last materialized date. Idempotent.
- Triggers: (a) inside `NewRecurringRule::insert` for create-time backfill, (b) the `POST /workspaces/{ws}/recurring/materialize` endpoint which iterates every active rule with `up_to = today`. No background daemon.

### Query methods
- `WorkspaceRef::recurring_rules(conn)` — active rules, ordered by `dtstart` ascending

## Soft Deletes

Entities with `deleted_at: Option<NaiveDateTime>`. All queries filter `deleted_at.is_null()`. Account soft-delete rejects if active children exist OR if account has postings on non-deleted entries. Baselines use hard delete (no deleted_at).

## Insert Pattern

Each entity uses a two-struct pattern:
- `NewFoo` — public builder with constructors and chainable methods
- `NewFooRow` — private `#[derive(Insertable)]` struct matching DB columns
- `insert(self, conn)` converts builder → row, does validation, returns loaded entity

## Testing

- Unit tests: `#[cfg(test)] mod tests` in each domain file, use `setup_test_db()` for in-memory SQLite
- Integration tests: `tests/api.rs` — builds full router with `test_pool()`, uses `tower::oneshot` for HTTP requests
- Helper functions: `register()`, `setup_workspace()`, `send()`, `get()`, `post()`, `put()`, `delete()`
