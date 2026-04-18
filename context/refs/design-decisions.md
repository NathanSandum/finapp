# Design Decisions

Captured during the Draft phase design conversation.

## Core Philosophy

The app aims to **inform, not restrict**. Users should have maximum visibility into their financial picture — past, present, and projected future. The goal is awareness, not policing.

## Engineering Principle

**No speculative complexity.** Write good code for what's needed now. Migration difficulty is never a reason to add complexity early — for a single-user MVP, data migrations are acceptable and AI-assisted migration is viable. Add things when there's a concrete reason, not in anticipation of hypothetical future needs.

## Key Decisions

### Multi-tenancy + Sharing
- Many independent users, each with their own isolated financial data
- Users can optionally invite others to view or collaborate on their workspace (couples, family, eventually financial advisors)
- Modelled as a **workspace** with membership roles

### Account Model
- Flat account list — all accounts share the same entity with a `type` field (chequing, savings, TFSA, RRSP, brokerage, etc.)
- Account internals are a black box — the app tracks inflows/outflows, not internal holdings
- Inspired by: chequing, savings, TFSA, Questrade in current spreadsheet

### Transaction Model
- Transactions + line items — a transaction is a single event (e.g. a credit card charge), line items are the category breakdown within it
- Motivated by real-world ambiguity: a single purchase may span multiple categories
- A transfer between accounts is two transactions linked together

### Baselines (not Budgets)
- A baseline is a **reference amount** per category per period — used for variance and trend comparison, not as a spending limit
- Framing: "you spent 23% more on cafes than your baseline" not "you're over budget"
- Future: baselines will eventually be replaced or supplemented by computed projections (linear regression, stochastic models)
- Lifestyle inflation tracking: compare personal spending growth rate against CPI

### Membership Event Sourcing
- Membership state uses an **append-only events log** (`membership_events`) — invited, accepted, role_changed, removed, left
- A **current-state projection** (`workspace_members`) is maintained transactionally alongside each event write
- Permission checks always read from the projection, not the log
- This gives a full audit trail and allows state reconstruction at any point in time
- In SQLite: projection is a regular table updated in the same transaction as the event; in Postgres: could be a materialized view or the same pattern
- The at-least-one-owner invariant is enforced by checking the projection before writing a remove/leave event, within a single transaction

### Out of Scope (for initial kits)
- Projections, retirement planning, stochastic modelling — schema must accommodate these but they are not implemented yet
- Tax calculations
- Investment account internals (holdings, tickers)
- Bank/brokerage import (future feature)
- Inflation tracking (future feature)
