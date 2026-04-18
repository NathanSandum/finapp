# Cavekit Overview

## Philosophy

This app informs rather than restricts. Users gain maximum visibility into their financial picture — past, present, and eventually projected future — without judgement or policing.

## Domains

| Domain | File | Summary |
|--------|------|---------|
| Workspace | cavekit-workspace.md | Users, workspaces, membership, sharing |
| Ledger | cavekit-ledger.md | Accounts, transactions, line items, categories |
| Baselines | cavekit-baselines.md | Reference amounts per category per period for variance/trend |
| API | cavekit-api.md | REST endpoints, request/response contracts, error handling |

## Cross-Cutting Concerns

- **Workspace isolation:** all ledger and baseline data is scoped to a workspace — see cavekit-workspace.md R1
- **Soft deletes:** no financial data is hard-deleted — see cavekit-ledger.md R7
- **Future accommodation:** schema must not prevent adding projections, retirement planning, or inflation tracking later — see cavekit-ledger.md R8, cavekit-baselines.md R5
- **TDD:** every requirement maps to a test gate — tests are written before implementation
