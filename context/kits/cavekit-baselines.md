# Cavekit: Baselines

## Scope

Covers baseline amounts per category per period, and variance computation against actuals. A baseline is a reference point — not a limit or budget. It exists to give context to spending patterns, support trend analysis, and eventually serve as one input to projection models. All data is scoped to a workspace.

## Requirements

### R1: Baseline Periods
**Description:** A baseline is defined for a specific period (monthly or quarterly) and a specific category. It represents an expected or reference amount for that category in that period.
**Acceptance Criteria:**
- [ ] A baseline has a unique identifier, category, period type (`monthly` or `quarterly`), period start date, reference amount, and timestamps
- [ ] A baseline belongs to exactly one workspace
- [ ] Only one baseline per category per period start date is allowed within a workspace
- [ ] A baseline amount can be zero (representing an expected zero spend)
- [ ] A baseline can be updated without affecting historical variance computations
**Dependencies:** cavekit-workspace.md R2, cavekit-ledger.md R2

### R2: Variance Computation
**Description:** Variance is the difference between actual spending in a category for a period and the baseline for that category and period. It is informational — there is no concept of "over" or "under" budget.
**Acceptance Criteria:**
- [ ] Variance equals actual amount minus baseline amount for a given category and period
- [ ] Variance is computed on demand from current transaction data — it is not stored
- [ ] A category with no baseline returns actuals with no variance value (not zero variance)
- [ ] A category with no actuals but a baseline returns a variance equal to the negative of the baseline
- [ ] Variance is presented as both absolute amount and percentage difference
**Dependencies:** R1, cavekit-ledger.md R5, R6

### R3: Period Summaries
**Description:** A period summary aggregates actuals and baselines across all categories for a given period, enabling a full picture of the period at a glance.
**Acceptance Criteria:**
- [ ] A period summary returns, for each category with activity or a baseline: actual total, baseline amount, absolute variance, percentage variance
- [ ] Categories with actuals but no baseline are included with a null baseline
- [ ] Categories with a baseline but no actuals are included with zero actuals
- [ ] The summary is filterable by category
- [ ] Period summaries are available for monthly and quarterly periods
**Dependencies:** R1, R2


## Out of Scope
- Trend queries across multiple periods
- Automated or computed baselines
- Alerts or notifications when actuals diverge from baselines
- Restrictions, limits, or any concept of "over budget"

## Cross-References
- Depends on: cavekit-workspace.md R2, R5 — workspace scoping and isolation
- Depends on: cavekit-ledger.md R2 — categories defined in ledger domain
- Depends on: cavekit-ledger.md R5, R6 — balance and transaction queries used for actuals
- Depended on by: cavekit-api.md — baseline endpoints derive from these requirements
