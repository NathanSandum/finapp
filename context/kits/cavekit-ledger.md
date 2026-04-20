# Cavekit: Ledger

## Scope

Covers accounts, transactions, and postings using double-entry bookkeeping. Accounts are hierarchical and serve as both "accounts" and "categories" — there is no separate category concept. A transaction is a dated financial event with two or more postings that must sum to zero. All data is scoped to a workspace.

This design follows hledger conventions: expenses, income, assets, liabilities, and equity are all accounts in a unified tree.

## Requirements

### R1: Accounts

**Description:** An account is a node in a hierarchical tree. Every account has a type derived from its root ancestor. Accounts replace the former separate "category" concept — an expense category like "Cafés" is simply the account `Expenses:Food:Cafés`.

**Acceptance Criteria:**
- [ ] An account has a unique identifier, name, optional parent account, currency, and timestamps
- [ ] An account belongs to exactly one workspace
- [ ] An account's full name is the colon-delimited path from root to leaf (e.g. `Assets:Bank:Chequing`)
- [ ] Two accounts with the same parent cannot share the same name segment
- [ ] An account's type is determined by its root ancestor and is one of: `asset`, `liability`, `equity`, `revenue`, `expense`
- [ ] Five root accounts are created automatically when a workspace is created: `Assets`, `Liabilities`, `Equity`, `Revenue`, `Expenses`
- [ ] A default set of sub-accounts is created under the root accounts when a workspace is created (e.g. `Assets:Chequing`, `Assets:Savings`, `Expenses:Food`, `Expenses:Rent`, `Revenue:Employment`)
- [ ] Accounts can be nested to arbitrary depth
- [ ] An account's balance is the sum of its own postings plus all descendant account postings

**Dependencies:** cavekit-workspace.md R2

### R2: Transactions

**Description:** A transaction is a dated financial event. It contains two or more postings. The sum of all posting amounts in a transaction must equal zero (double-entry invariant).

**Acceptance Criteria:**
- [ ] A transaction has a unique identifier, date, description, and timestamps
- [ ] A transaction belongs to exactly one workspace
- [ ] A transaction has at least two postings
- [ ] The sum of all posting amounts in a transaction equals zero
- [ ] A transaction can be marked as cleared or pending

**Dependencies:** R1

### R3: Postings

**Description:** A posting is a single debit or credit entry within a transaction. Each posting references one account and has an amount. Positive amounts are debits, negative amounts are credits.

**Acceptance Criteria:**
- [ ] A posting has a unique identifier, amount, account reference, optional note, and timestamps
- [ ] A posting belongs to exactly one transaction
- [ ] A posting amount must be non-zero
- [ ] A posting references exactly one account
- [ ] Creating, updating, or deleting a posting that would violate the zero-sum invariant is prevented

**Dependencies:** R1, R2

### R4: Balance Computation

**Description:** An account's balance at any point in time is computed from its postings and those of all descendant accounts. The balance reflects all cleared postings up to and including a given date.

**Acceptance Criteria:**
- [ ] The current balance of an account equals the sum of all cleared posting amounts for that account and its descendants
- [ ] A balance as-of a given date reflects only postings on or before that date
- [ ] Pending postings (from pending transactions) are excluded from the cleared balance but visible separately
- [ ] Balance computation is consistent — the same inputs always produce the same result
- [ ] The fundamental equation holds: Assets = Liabilities + Equity + (Revenue - Expenses)

**Dependencies:** R2, R3

### R5: Balance Sheet

**Description:** A balance sheet reports the financial position at a point in time. It includes only accounts under `Assets`, `Liabilities`, and `Equity`. Net income (Revenue - Expenses) is shown as a component of Equity.

**Acceptance Criteria:**
- [ ] Balance sheet includes all `Assets`, `Liabilities`, and `Equity` accounts with balances
- [ ] Net income (sum of `Revenue` postings minus sum of `Expenses` postings) is included as `Equity:Retained Earnings` or a similar line
- [ ] Balance sheet balances: total assets = total liabilities + total equity (including net income)
- [ ] Balance sheet can be computed as-of any date

**Dependencies:** R4

### R6: Income Statement

**Description:** An income statement reports revenue and expenses over a period. It includes only accounts under `Revenue` and `Expenses`.

**Acceptance Criteria:**
- [ ] Income statement includes all `Revenue` and `Expenses` accounts with activity in the period
- [ ] Income statement is computed for a date range (start date to end date, inclusive)
- [ ] Net income equals total revenue minus total expenses for the period
- [ ] Accounts with zero activity in the period are omitted

**Dependencies:** R4

### R7: Transaction Queries

**Description:** Transactions can be queried by date range, account (including descendants), and cleared status. Results are paginated.

**Acceptance Criteria:**
- [ ] Transactions can be filtered by one or more accounts (matching any posting's account or its ancestors)
- [ ] Transactions can be filtered by date range (start date, end date, inclusive)
- [ ] Transactions can be filtered by cleared status
- [ ] Query results are paginated with a configurable page size (default 50, max 200)
- [ ] Results are ordered by date descending by default

**Dependencies:** R2, R3

### R8: Soft Deletes

**Description:** No financial data is permanently deleted. Accounts, transactions, and postings are soft-deleted — they retain their data but are excluded from normal queries.

**Acceptance Criteria:**
- [ ] Soft-deleting an account hides it from account listings and balance computations
- [ ] Soft-deleting an account with descendant accounts is prevented unless all descendants are already soft-deleted
- [ ] Soft-deleting a transaction hides it from transaction queries and balance computations
- [ ] Soft-deleted records are recoverable
- [ ] Soft-deleted records retain all their associated data
- [ ] Queries for active records never return soft-deleted records unless explicitly requested

**Dependencies:** R1, R2

## Out of Scope
- Investment holdings, tickers, or brokerage-specific transaction fields
- Recurring transaction automation (future)
- Bank/brokerage import (future)
- Multi-currency conversion (future — currency is stored, conversion is not)
- Receipt or attachment storage
- Period closing entries (net income rolls up implicitly without explicit close)

## Cross-References
- Depends on: cavekit-workspace.md R2, R5 — all data is workspace-scoped and isolated
- Depended on by: cavekit-baselines.md — baselines reference accounts (formerly categories) from this domain
- Depended on by: cavekit-api.md — all ledger endpoints derive from these requirements
