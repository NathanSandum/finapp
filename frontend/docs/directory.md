# Frontend Directory Structure

```
frontend/
├── CLAUDE.md                          # Agent entry point
├── docs/
│   ├── directory.md                   # This file
│   ├── architecture.md                # Component design, state, API patterns
│   └── refactoring-todos.md           # Tech debt and decomposition targets
├── src/
│   ├── app.d.ts                       # TypeScript ambient declarations (App.Locals, etc.)
│   ├── routes/
│   │   ├── +layout.svelte             # Root layout — imports CSS, calls loadToken()
│   │   ├── +layout.ts                 # Config: ssr=false, prerender=true
│   │   ├── layout.css                 # Global Tailwind imports
│   │   ├── +page.svelte               # Main app shell — sidebar tabs, auth bootstrap, workspace init
│   │   └── login/
│   │       └── +page.svelte           # Login/register form with mode toggle
│   └── lib/
│       ├── index.ts                   # Barrel export (unused)
│       ├── api.svelte.ts              # REST API client — auth, workspaces, accounts, entries, reports, baselines
│       ├── types.ts                   # Shared type definitions (Account, Entry, Posting, reports, etc.)
│       ├── format.ts                  # Shared formatAmount(cents) utility
│       ├── period.ts                  # Current month period calculation
│       ├── import.ts                  # CSV import logic — parsing, detection, entry generation (pure functions)
│       ├── import.test.ts             # Unit tests for import.ts
│       └── components/
│           ├── ImportView.svelte      # CSV import orchestrator — state machine + step routing
│           ├── ImportUploadStep.svelte # Step 1: file picker + year input
│           ├── ImportMappingStep.svelte# Step 2: column type dropdowns + account selectors
│           ├── ImportPreviewStep.svelte# Step 3: stats cards + sample entries
│           ├── ImportProgressStep.svelte# Step 4: progress bar during import
│           ├── ImportDoneStep.svelte   # Step 5: success count + error list
│           ├── AccountSelect.svelte   # Reusable account dropdown with "create new" option
│           ├── FundingSelect.svelte   # Reusable funding account dropdown
│           ├── IncomeView.svelte      # Income statement for current month
│           ├── CashFlowView.svelte    # Cash flow — combined revenue/expense table
│           ├── BalanceSheetView.svelte # Balance sheet — assets/liabilities/equity sections
│           └── EntriesView.svelte     # Transaction ledger with delete
├── static/
│   └── robots.txt
├── svelte.config.js                   # SvelteKit config — static adapter, Svelte 5 runes
├── vite.config.ts                     # Vite config with Vitest for browser testing
├── tsconfig.json                      # TypeScript strict mode
├── eslint.config.js                   # ESLint + Svelte plugin
└── .prettierrc                        # Prettier config
```

## File Size Reference

| File | Lines | Complexity |
|------|-------|------------|
| ImportView.svelte | ~200 | Medium — orchestrator with state machine + import execution |
| ImportMappingStep.svelte | ~140 | Medium — column mapping UI with type-dependent forms |
| ImportPreviewStep.svelte | ~110 | Low — read-only preview |
| import.ts | ~315 | Medium — pure functions, well-tested |
| import.test.ts | 401 | Tests for import.ts |
| api.svelte.ts | ~200 | Low — thin fetch wrapper |
| types.ts | ~65 | Low — shared type definitions |
| IncomeView.svelte | ~130 | Low |
| BalanceSheetView.svelte | ~115 | Low |
| CashFlowView.svelte | ~115 | Low |
| EntriesView.svelte | ~100 | Low |
| login/+page.svelte | 112 | Low |
| +page.svelte | 106 | Low |
