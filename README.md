# FinApp

A double-entry personal finance application built with Rust, SvelteKit, and Tauri.

## Overview

FinApp follows standard accounting principles — every transaction has balanced debits and credits across a hierarchical chart of accounts. It supports multiple workspaces with role-based access, financial reporting (balance sheet, income statement), budgeting with variance tracking, and CSV import.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, Diesel, SQLite |
| Frontend | SvelteKit 2, Svelte 5, TypeScript, Tailwind CSS 4 |
| Desktop | Tauri 2 |
| Auth | Argon2 password hashing, session tokens |

## Features

- **Double-entry bookkeeping** — entries with two or more zero-sum postings
- **Hierarchical accounts** — tree structure with five root types (Assets, Liabilities, Equity, Revenue, Expenses) and 18 default accounts
- **Balance sheet & income statement** — date-filtered financial reports with rolled-up balances
- **Baselines (budgets)** — monthly budget per account with variance calculation
- **CSV import** — multi-step wizard with column mapping
- **Multi-user workspaces** — role-based access (Viewer, Editor, Owner)
- **Soft deletes** — entries and accounts are preserved in the database

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (for desktop builds)

### Backend

```bash
cd backend
cargo run    # starts API server on http://127.0.0.1:3000
```

### Frontend

```bash
cd frontend
npm install
npm run dev  # starts dev server on http://localhost:5173
```

### Desktop

```bash
cd desktop
cargo tauri dev    # launches Tauri window with frontend dev server
cargo tauri build  # bundles for distribution
```

## Testing

```bash
# Backend (integration + unit tests)
cd backend && cargo test

# Frontend
cd frontend && npm run test
```

## Project Structure

```
finapp/
├── backend/          # Rust API server
│   ├── src/
│   │   ├── domain/   # Account, Entry, Baseline, Report, User, Workspace
│   │   ├── routes/   # API endpoints
│   │   └── auth.rs   # Authentication extractors
│   ├── migrations/   # Database migrations
│   └── tests/        # Integration tests
├── frontend/         # SvelteKit SPA
│   └── src/
│       ├── routes/   # Pages and layouts
│       └── lib/      # API client, components, types
└── desktop/          # Tauri shell
```

## License

All rights reserved.
