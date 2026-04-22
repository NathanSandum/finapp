# Frontend Architecture

## Stack

- **SvelteKit 2** with `adapter-static` — builds to pure client-side SPA
- **Svelte 5** with runes enabled globally (`compilerOptions.runes: true`)
- **Tailwind CSS 4** with forms + typography plugins
- **Vitest** for unit tests (browser mode)
- **TypeScript** strict mode

## Routing

Two routes, both client-side only (`ssr=false`):

| Route | File | Purpose |
|-------|------|---------|
| `/` | `+page.svelte` | Main dashboard shell |
| `/login` | `login/+page.svelte` | Auth form |

Navigation within `/` is tab-based using local `$state` — no URL state, no SvelteKit navigation. Refresh always returns to Income tab.

## Component Model

Root page (`+page.svelte`) handles:
1. Auth bootstrap — auto-registers/logs in `dev@finapp.local` on mount
2. Workspace init — creates "My Finances" if none exist
3. Tab switching between 5 views

Each view component receives `workspaceId: string` as its only prop. Components are independent — no shared state, no cross-component communication.

```
+page.svelte (shell)
├── IncomeView            — GET /reports/income-statement
├── CashFlowView          — GET /reports/income-statement (same endpoint, different display)
├── BalanceSheetView       — GET /reports/balance-sheet
├── EntriesView            — GET /entries + GET /accounts
└── ImportView (orchestrator — state machine + import execution)
    ├── ImportUploadStep   — file picker, year input
    ├── ImportMappingStep  — column type mapping + account selection
    │   ├── AccountSelect  — reusable dropdown with "create new" option
    │   └── FundingSelect  — reusable funding account dropdown
    ├── ImportPreviewStep  — stats + sample entries
    ├── ImportProgressStep — progress bar
    └── ImportDoneStep     — results + errors
```

## State Management

**No external store library.** All state uses Svelte 5 runes:

- `$state` — local reactive state in components
- `$derived` / `$derived.by` — computed values (e.g., `leafAccounts`, `previewStats`)
- `$effect` — side effects (auth redirect in login page)
- `$props` — component inputs (just `workspaceId` everywhere)

**Global state:** Only auth token, managed by `api.svelte.ts`:
- `setToken(t)` — sets reactive `$state` + writes `localStorage`
- `loadToken()` — reads from `localStorage` into `$state`
- `getToken()` — returns current value

## API Client (`api.svelte.ts`)

Thin wrapper around `fetch`. Base URL: `/api`. Bearer token auth.

Pattern for every endpoint:
```typescript
export async function doThing(workspace: string, ...args) {
  const res = await request('METHOD', `/workspaces/${workspace}/path`, body?);
  if (!res.ok) throw await res.json();
  return res.json();
}
```

**Domains:** auth (register/login/logout), workspaces (list/create), accounts (list/create/balance), entries (list/get/create/delete), reports (balance-sheet/income-statement), baselines (list/create/delete/variance).

All API response types live in `$lib/types.ts` (Account, Entry, Posting, PostingInput, AccountBalance, IncomeStatement, BalanceSheet). Import-specific types (MappingType, ColumnMapping, DateRow, EntryInput) live in `$lib/import.ts`.

## CSV Import Flow

Split across two files:

**`import.ts`** — Pure functions (no API calls, no DOM):
- CSV parsing: `parseCSVLine`, `parseDate`, `parseDollars`, `parseCSV`
- Auto-detection: `autoDetectMappings` — guesses column types from header names
- Entry generation: `buildOpeningBalanceEntry`, `buildActivityEntries`
- Account collection: `collectNewAccounts`

**`ImportView.svelte`** — Orchestration UI (594 lines, 5-step wizard):
1. **Upload** — file picker + year selector
2. **Mapping** — column type dropdowns + account selectors
3. **Preview** — stats + sample entries
4. **Importing** — progress bar while creating accounts/entries via API
5. **Done** — success count + error list

CSV format expected: first column is day (`"Jan  1"` or `"Start"`), remaining columns are dollar amounts. Headers determine account names.

## Styling

Tailwind utility classes only. Dark theme: `bg-gray-950` base, `gray-800` borders, `gray-900` cards. Color semantics: `emerald-400` for positive amounts, `red-400` for negative/expenses. No component library.

## Amount Handling

All monetary values stored as **integer cents**. Shared `formatAmount(cents)` in `$lib/format.ts` formats to `$1,234.56`. Period date calculation shared via `$lib/period.ts`.
