# Backend

Rust + Axum + Diesel + SQLite. Double-entry personal finance app.

## Agent Docs

Read these before touching code — they should be sufficient without reading source files:

- `docs/directory.md` — file tree with purpose annotations
- `docs/domain-model.md` — types, relationships, patterns, conventions
- `docs/api.md` — routes, request/response shapes, auth flow
- `docs/current-state.md` — what's done, what's stubbed, roadmap position

## Conventions

- TDD: tests first, then implementation
- All IDs are Text UUIDs
- Database connection is always the last parameter
- Use `foo.rs` not `foo/mod.rs` for modules
- Never abbreviate variable names (no ws, cat, acct, txn)
- Changes go through PRs, never push directly to main
- Build for now — no speculative complexity
