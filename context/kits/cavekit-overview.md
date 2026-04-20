# Cavekit Overview

## Philosophy

This app informs rather than restricts. Users gain maximum visibility into their financial picture — past, present, and eventually projected future — without judgement or policing.

## Domains

| Domain | File | Summary |
|--------|------|---------|
| Workspace | cavekit-workspace.md | Users, workspaces, membership, sharing |
| Ledger | cavekit-ledger.md | Hierarchical accounts (double-entry), transactions, postings, balance sheet, income statement |
| Baselines | cavekit-baselines.md | Reference amounts per category per period for variance/trend |
| API | cavekit-api.md | REST endpoints, request/response contracts, error handling |

## Cross-Cutting Concerns

- **Double-entry bookkeeping:** every transaction balances to zero. No separate "category" concept — categories are accounts in the hierarchy (e.g. `Expenses:Food:Cafés`)
- **Hierarchical accounts:** accounts form a tree rooted at five top-level types: Assets, Liabilities, Equity, Revenue, Expenses
- **Workspace isolation:** all ledger and baseline data is scoped to a workspace — see cavekit-workspace.md R1
- **Soft deletes:** no financial data is hard-deleted — see cavekit-ledger.md R8
- **Future accommodation:** schema must not prevent adding projections, retirement planning, or inflation tracking later
- **TDD:** every requirement maps to a test gate — tests are written before implementation
