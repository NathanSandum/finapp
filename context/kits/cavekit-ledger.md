# Cavekit: Ledger

## Scope

Covers accounts, categories, transactions, and line items. This is the core financial data model. An account is a named balance with a type. A transaction is a dated financial event against one or more accounts. Line items are the category breakdown within a transaction. All data is scoped to a workspace.

## Requirements

### R1: Accounts
**Description:** An account represents a financial account with a type label. The internal mechanics of the account (e.g. investment holdings) are out of scope — the app tracks inflows and outflows only.
**Acceptance Criteria:**
- [ ] An account has a unique identifier, name, type, currency, and timestamps
- [ ] Account types include at minimum: `chequing`, `savings`, `tfsa`, `rrsp`, `brokerage`, `credit`, `other`
- [ ] An account belongs to exactly one workspace
- [ ] Two accounts in the same workspace can share the same type but not the same name
- [ ] An account has a computed current balance derived from its transactions
**Dependencies:** cavekit-workspace.md R2

### R2: Categories
**Description:** A category is a named label used to classify line items. Categories are scoped to a workspace and can be customised by workspace members.
**Acceptance Criteria:**
- [ ] A category has a unique identifier, name, optional description, and timestamps
- [ ] A category belongs to exactly one workspace
- [ ] Two categories in the same workspace cannot share the same name
- [ ] Deleting a category that has associated line items is prevented unless those line items are reassigned or the category is soft-deleted
- [ ] A default set of categories is created when a workspace is created (e.g. employment income, rent, groceries, dining, transit, utilities, entertainment, other)
**Dependencies:** cavekit-workspace.md R2

### R3: Transactions
**Description:** A transaction is a dated financial event. It references one primary account (the account the money moves from or to) and has one or more line items. A transaction may optionally reference a second account to represent a transfer.
**Acceptance Criteria:**
- [ ] A transaction has a unique identifier, date, description, primary account, total amount, and timestamps
- [ ] A transaction belongs to exactly one workspace
- [ ] A transaction has at least one line item
- [ ] The sum of line item amounts equals the transaction total amount
- [ ] A transfer transaction references two accounts within the same workspace
- [ ] A transaction can be marked as cleared or pending
**Dependencies:** R1

### R4: Line Items
**Description:** A line item is a categorised amount within a transaction. It represents a portion of the transaction total assigned to a specific category.
**Acceptance Criteria:**
- [ ] A line item has a unique identifier, amount, category, optional note, and timestamps
- [ ] A line item belongs to exactly one transaction
- [ ] A line item amount must be non-zero
- [ ] A line item references exactly one category
- [ ] Updating a line item amount such that line items no longer sum to the transaction total is prevented
**Dependencies:** R2, R3

### R5: Balance Computation
**Description:** An account's balance at any point in time is computed from its transactions. The balance reflects all cleared transactions up to and including a given date.
**Acceptance Criteria:**
- [ ] The current balance of an account equals the sum of all cleared transaction amounts for that account
- [ ] A balance as-of a given date reflects only transactions on or before that date
- [ ] Pending transactions are excluded from the cleared balance but visible separately
- [ ] Balance computation is consistent — the same inputs always produce the same result
**Dependencies:** R3

### R6: Transaction Queries
**Description:** Transactions can be queried by date range, account, category, and cleared status. Results are paginated.
**Acceptance Criteria:**
- [ ] Transactions can be filtered by one or more accounts
- [ ] Transactions can be filtered by date range (start date, end date, inclusive)
- [ ] Transactions can be filtered by one or more categories (via their line items)
- [ ] Transactions can be filtered by cleared status
- [ ] Query results are paginated with a configurable page size (default 50, max 200)
- [ ] Results are ordered by date descending by default
**Dependencies:** R3, R4

### R7: Soft Deletes
**Description:** No financial data is permanently deleted. Accounts, transactions, line items, and categories are soft-deleted — they retain their data but are excluded from normal queries.
**Acceptance Criteria:**
- [ ] Soft-deleting an account hides it from account listings and balance computations
- [ ] Soft-deleting a transaction hides it from transaction queries and balance computations
- [ ] Soft-deleted records are recoverable
- [ ] Soft-deleted records retain all their associated data
- [ ] Queries for active records never return soft-deleted records unless explicitly requested
**Dependencies:** R1, R3

## Out of Scope
- Investment holdings, tickers, or brokerage-specific transaction fields
- Recurring transaction automation (future)
- Bank/brokerage import (future)
- Multi-currency conversion (future — currency is stored, conversion is not)
- Receipt or attachment storage

## Cross-References
- Depends on: cavekit-workspace.md R2, R5 — all data is workspace-scoped and isolated
- Depended on by: cavekit-baselines.md — baselines reference categories from this domain
- Depended on by: cavekit-api.md — all ledger endpoints derive from these requirements
