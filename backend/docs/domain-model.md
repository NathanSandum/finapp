# Domain Model

## Entity Relationships

```
User ──< WorkspaceUser >── Workspace
              │                 │
              │                 ├──< Account (tree via parent_id)
              │                 │       └── AccountType: asset|liability|equity|revenue|expense
              │                 │
              │                 └──< Transaction ──< Posting >── Account
              │                                     (amount in cents, zero-sum per transaction)
         MembershipEvent (append-only log)
```

## Ref Pattern

Every entity has a `FooRef` newtype wrapping `String` (UUID v4 as text). Derived via `DieselNewType`. Used for type-safe foreign keys and path parameters.

- `UserRef`, `WorkspaceRef`, `AccountRef`, `TransactionRef`, `PostingRef`
- Each entity has `as_ref() -> &FooRef` and `from_ref(ref, conn) -> Result<Option<Foo>>`
- Connection parameter is always last

## Enums Stored as Text

`AccountType` and `Role` both follow same pattern:
- `as_str() -> &'static str` / `from_str(s) -> Option<Self>`
- `FromSql<Text, Sqlite>` / `ToSql<Text, Sqlite>` impls
- Queryable impl for use as Diesel column type
- Serde with `rename_all = "snake_case"` (AccountType) or manual (Role)

### AccountType
`Asset`, `Liability`, `Equity`, `Revenue`, `Expense`

### Role
`Viewer` < `Editor` < `Owner` — implements `Ord` for role comparison. `WorkspaceMember::require_role()` checks minimum role.

## Account Hierarchy

Accounts form a tree rooted at five types. `parent_id` is nullable — NULL means root.

- `account_type` is denormalized on every row, inherited from root ancestor at insert time
- Uniqueness: `UNIQUE(workspace_id, parent_id, name)` in DB for siblings. Root uniqueness (NULL parent) enforced in application code (SQLite treats NULL as distinct)
- `NewAccount::root(workspace, name, account_type)` — creates root, checks name uniqueness manually
- `NewAccount::child(parent, name)` — inherits workspace, account_type, currency from parent
- `.currency(str)` builder method overrides default "CAD"

### Default Tree (created on workspace creation)
```
Assets: Chequing, Savings, TFSA, RRSP
Liabilities: Credit Card
Equity: (no children)
Revenue: Employment
Expenses: Food, Rent, Groceries, Dining, Transit, Utilities, Entertainment
```
Total: 5 roots + 13 children = 18 accounts

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

## Transactions & Postings (Double-Entry)

Every transaction has 2+ postings that sum to zero. Amounts in integer cents (i32).

### Transaction
- Fields: id, workspace, date (TEXT "YYYY-MM-DD"), description, cleared (bool), deleted_at, timestamps
- `postings(conn)` — all postings for this transaction
- `for_account(account_ref, conn)` — transactions with posting to this account, excluding soft-deleted
- `for_account_tree(account, conn)` — same but includes descendant accounts
- `for_workspace(workspace_ref, conn)` — all workspace transactions

### Posting
- Fields: id, transaction (TransactionRef), account (AccountRef), amount (i32 cents), note, timestamps
- No independent CRUD — created atomically with transaction

### NewTransaction Builder
- `NewTransaction::new(workspace, date, description, postings: Vec<PostingInput>)`
- `.cleared(bool)` chainable
- `insert(conn)` validates: min 2 postings, sum == 0, no zero amounts. Atomic insert.
- `PostingInput::new(account, amount)` with `.note(str)` chainable

### Balance Methods (on Account)
- `balance(conn) -> i64` — sum of postings for this account on cleared, non-deleted transactions
- `balance_with_descendants(conn) -> i64` — includes all descendant account postings
- `balance_as_of(date, conn) -> i64` — filtered by transaction date <= date
- Return i64 (not i32) to handle overflow from summing many postings

### Sign Convention
Positive postings = debits, negative = credits. Revenue/liability accounts have negative normal balance. Sum of all account balances across entire workspace = 0.

## Soft Deletes

Entities with `deleted_at: Option<NaiveDateTime>`. All queries filter `deleted_at.is_null()`. Account soft-delete rejects if active children exist OR if account has postings on non-deleted transactions.

## Insert Pattern

Each entity uses a two-struct pattern:
- `NewFoo` — public builder with constructors and chainable methods
- `NewFooRow` — private `#[derive(Insertable)]` struct matching DB columns
- `insert(self, conn)` converts builder → row, does validation, returns loaded entity

## Testing

- Unit tests: `#[cfg(test)] mod tests` in each domain file, use `setup_test_db()` for in-memory SQLite
- Integration tests: `tests/api.rs` — builds full router with `test_pool()`, uses `tower::oneshot` for HTTP requests
- Helper functions: `register()`, `setup_workspace()`, `send()`, `get()`, `post()`, `put()`, `delete()`
