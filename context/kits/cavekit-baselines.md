# Cavekit: Baselines

## Scope

Covers baseline amounts per account per period, and variance computation against actuals. A baseline is a reference point — not a limit or budget. It exists to give context to spending patterns, support trend analysis, and eventually serve as one input to projection models. All data is scoped to a workspace.

Baselines are typically set on `Expenses` and `Revenue` accounts but can be set on any account.

## Requirements

### R1: Baseline Periods
**Description:** A baseline is defined for a specific period (monthly or quarterly) and a specific account. It represents an expected or reference amount for that account in that period.
**Acceptance Criteria:**
- [ ] A baseline has a unique identifier, account, period type (`monthly` or `quarterly`), period start date, reference amount, and timestamps
- [ ] A baseline belongs to exactly one workspace
- [ ] Only one baseline per account per period start date is allowed within a workspace
- [ ] A baseline amount can be zero (representing an expected zero spend)
- [ ] A baseline can be updated without affecting historical variance computations
**Dependencies:** cavekit-workspace.md R2, cavekit-ledger.md R1

### R2: Variance Computation
**Description:** Variance is the difference between actual posting totals for an account in a period and the baseline for that account and period. It is informational — there is no concept of "over" or "under" budget.
**Acceptance Criteria:**
- [ ] Variance equals actual amount minus baseline amount for a given account and period
- [ ] Variance is computed on demand from current posting data — it is not stored
- [ ] An account with no baseline returns actuals with no variance value (not zero variance)
- [ ] An account with no actuals but a baseline returns a variance equal to the negative of the baseline
- [ ] Variance is presented as both absolute amount and percentage difference
**Dependencies:** R1, cavekit-ledger.md R4, R7

### R3: Period Summaries
**Description:** A period summary aggregates actuals and baselines across all accounts for a given period, enabling a full picture of the period at a glance.
**Acceptance Criteria:**
- [ ] A period summary returns, for each account with activity or a baseline: actual total, baseline amount, absolute variance, percentage variance
- [ ] Accounts with actuals but no baseline are included with a null baseline
- [ ] Accounts with a baseline but no actuals are included with zero actuals
- [ ] The summary is filterable by account (including descendants)
- [ ] Period summaries are available for monthly and quarterly periods
**Dependencies:** R1, R2


## Out of Scope
- Trend queries across multiple periods
- Automated or computed baselines
- Alerts or notifications when actuals diverge from baselines
- Restrictions, limits, or any concept of "over budget"

## Cross-References
- Depends on: cavekit-workspace.md R2, R5 — workspace scoping and isolation
- Depends on: cavekit-ledger.md R1 — accounts (formerly categories) defined in ledger domain
- Depends on: cavekit-ledger.md R4, R7 — balance and transaction queries used for actuals
- Depended on by: cavekit-api.md — baseline endpoints derive from these requirements
