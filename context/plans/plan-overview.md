# Plan Overview

## Build Sites

| File | Scope |
|------|-------|
| `build-site.md` | Full MVP — all waves from foundation to frontend (complete) |
| `build-double-entry.md` | Double-entry rewrite — hierarchical accounts, postings, reports |

## Double-Entry Rewrite — Wave Summary

| Wave | Name | Depends On |
|------|------|-----------|
| 1 | Account Hierarchy | — |
| 2 | Transactions & Postings | Wave 1 |
| 3 | Reports & Baselines | Wave 2 |
| 4 | API Routes | Wave 2, Wave 3 |
| 5 | Frontend Updates | Wave 4 |

## Kit Coverage

| Kit | Covered By |
|-----|-----------|
| cavekit-ledger.md R1 | Wave 1 |
| cavekit-ledger.md R2, R3 | Wave 2 |
| cavekit-ledger.md R4, R7, R8 | Wave 2 |
| cavekit-ledger.md R5, R6 | Wave 3 |
| cavekit-baselines.md | Wave 3 |
| cavekit-api.md R5 | Wave 4 |
| cavekit-api.md R6 | Wave 4 |
| cavekit-api.md R7 | Wave 4 |
| cavekit-api.md R8 | Wave 4 |
