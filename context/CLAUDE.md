# FinApp Context Root

This directory contains all agent-facing context for the finapp project. Navigate via the tier structure below — read only what your task requires.

## Tier Structure

| Tier | Directory | Purpose |
|------|-----------|---------|
| 1 | `refs/` | Source material — design decisions, reference docs. Read-only input. |
| 2 | `kits/` | WHAT to build — implementation-agnostic requirements with acceptance criteria |
| 3 | `plans/` | HOW to build — framework-specific task graphs, Axum + SQLite + SvelteKit |
| 4 | `impl/` | WHAT WAS DONE — per-domain progress tracking, updated after every session |

## Entry Points

- Requirements: start at `kits/cavekit-overview.md`
- Implementation plan: start at `plans/plan-overview.md`
- Progress: start at `impl/impl-overview.md`
- Design decisions and rationale: `refs/design-decisions.md`

## Stack

- **Backend:** Rust, Axum, sqlx, SQLite (local) / Postgres (future web)
- **Desktop:** Tauri (Rust shell + SvelteKit frontend)
- **Web frontend:** SvelteKit
- **Testing:** Rust `#[cfg(test)]` unit tests + integration tests; Vitest for frontend
- **Workflow:** TDD (tests before implementation), PR-based, no speculative complexity
