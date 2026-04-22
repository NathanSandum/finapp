# Frontend

SvelteKit 2 + Svelte 5 (runes) + Tailwind CSS 4. Static site adapter (no SSR).

## Agent Docs

Read these before touching code — they should be sufficient without reading source files:

- `docs/directory.md` — file tree with purpose annotations
- `docs/architecture.md` — component structure, state management, API patterns
- `docs/refactoring-todos.md` — known tech debt and decomposition targets

## Conventions

- Svelte 5 runes: `$state`, `$derived`, `$derived.by`, `$effect`, `$props`
- All amounts in cents (integer). Display with `formatAmount(cents)`.
- Components receive `workspaceId` as prop from root page
- API calls go through `$lib/api.svelte.ts` — never raw `fetch`
- Tailwind for all styling — no CSS files except `layout.css`
- Dark theme (gray-950 background) throughout
- TDD: tests first, then implementation
- Changes go through PRs, never push directly to main
- Never abbreviate variable names
