# Frontend Refactoring TODOs

## High Priority

No high-priority items remaining.

## Medium Priority

### TODO: Add URL-based tab state

Active tab is local `$state` — lost on refresh, not shareable, no browser back/forward. Use SvelteKit's URL query params or hash routing so `/?tab=entries` works.

### TODO: Add component tests

Only `import.ts` has tests (401 lines, good coverage). No component tests exist. Priority targets:
- `ImportView.svelte` — wizard flow, mapping validation, error states
- `EntriesView.svelte` — delete confirmation, empty state
- `login/+page.svelte` — mode toggle, error display

## Low Priority

### TODO: Decouple auth from root page

`+page.svelte` auto-registers `dev@finapp.local` on mount — hardcoded dev credentials. This should:
- Use the `/login` route properly
- Remove hardcoded credentials
- Handle token expiry gracefully (currently silently fails)

### TODO: Add pagination to EntriesView

Currently loads all entries into memory. Works at small scale but won't survive real usage. Backend may need pagination support too.

### TODO: Add API response caching

Each tab re-fetches on every tab switch (components mount/unmount). Consider:
- Caching at the API layer
- Or keeping components alive with Svelte's `{#key}` / conditional display instead of conditional render

### TODO: Add error boundaries

Most components swallow errors silently (`catch { /* no data yet */ }`). Add visible error states with retry buttons.
