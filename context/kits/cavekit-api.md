# Cavekit: API

## Scope

Covers the REST API surface: endpoint contracts, authentication, error handling, and pagination. The API serves both the desktop (Tauri) and web (SvelteKit) frontends. All endpoints require authentication and workspace membership appropriate to the operation.

## Requirements

### R1: Authentication
**Description:** All API endpoints (except registration and login) require a valid session token. Tokens are issued on login and expire after a configurable duration.
**Acceptance Criteria:**
- [ ] A request without a token receives a 401 response
- [ ] A request with an invalid or expired token receives a 401 response
- [ ] A valid token grants access to only the workspaces the user is a member of
- [ ] Tokens can be explicitly invalidated (logout)
**Dependencies:** cavekit-workspace.md R1

### R2: Authorisation
**Description:** All workspace-scoped endpoints enforce the caller's membership role. Insufficient role returns a 403.
**Acceptance Criteria:**
- [ ] A `viewer` attempting a write operation receives a 403 response
- [ ] A non-member attempting to access any workspace resource receives a 403 (or 404) response
- [ ] Role checks are enforced server-side — the client role claim is never trusted alone
**Dependencies:** cavekit-workspace.md R3

### R3: Error Responses
**Description:** All error responses follow a consistent structure so clients can handle them uniformly.
**Acceptance Criteria:**
- [ ] Error responses include a machine-readable `code` field and a human-readable `message` field
- [ ] Validation errors include a `fields` array identifying which inputs failed and why
- [ ] 404 responses do not distinguish between "does not exist" and "exists but forbidden" for workspace-scoped resources
- [ ] 500 responses do not leak internal error details to the client
**Dependencies:** None

### R4: Workspace Endpoints
**Description:** Endpoints to create, read, update, and manage workspaces and memberships.
**Acceptance Criteria:**
- [ ] `POST /workspaces` creates a workspace; creator becomes owner
- [ ] `GET /workspaces` returns all workspaces the authenticated user is a member of
- [ ] `GET /workspaces/:id` returns workspace details for a member
- [ ] `POST /workspaces/:id/invitations` sends an invitation (owner only)
- [ ] `POST /invitations/:token/accept` accepts an invitation
- [ ] `DELETE /workspaces/:id/members/:userId` removes a member (owner only)
**Dependencies:** R1, R2, cavekit-workspace.md

### R5: Account Endpoints
**Description:** Endpoints to manage hierarchical accounts within a workspace.
**Acceptance Criteria:**
- [ ] `POST /workspaces/:id/accounts` creates an account with optional parent (editor+)
- [ ] `GET /workspaces/:id/accounts` returns all active accounts as a tree with balances
- [ ] `GET /workspaces/:id/accounts/:accountId` returns account detail with balance (including descendants)
- [ ] `PATCH /workspaces/:id/accounts/:accountId` updates account name (editor+)
- [ ] `DELETE /workspaces/:id/accounts/:accountId` soft-deletes an account (editor+); fails if account has active descendants
**Dependencies:** R1, R2, cavekit-ledger.md R1, R4

### R6: Transaction Endpoints
**Description:** Endpoints to create, read, update, and delete transactions and their postings.
**Acceptance Criteria:**
- [ ] `POST /workspaces/:id/transactions` creates a transaction with postings (editor+); postings must sum to zero
- [ ] `GET /workspaces/:id/transactions` returns transactions with filter support (date range, account including descendants, cleared status), capped at 1000 rows, ordered by date descending
- [ ] `GET /workspaces/:id/transactions/:txId` returns a transaction with all postings
- [ ] `PATCH /workspaces/:id/transactions/:txId` updates transaction fields (editor+)
- [ ] `DELETE /workspaces/:id/transactions/:txId` soft-deletes a transaction (editor+)
- [ ] `POST /workspaces/:id/transactions/:txId/postings` adds a posting (editor+); zero-sum invariant must hold after
- [ ] `PATCH /workspaces/:id/transactions/:txId/postings/:postingId` updates a posting (editor+); zero-sum invariant must hold after
- [ ] `DELETE /workspaces/:id/transactions/:txId/postings/:postingId` removes a posting, provided at least two remain and zero-sum holds (editor+)
**Dependencies:** R1, R2, cavekit-ledger.md R2, R3, R7, R8

### R7: Report Endpoints
**Description:** Endpoints to retrieve balance sheet and income statement reports.
**Acceptance Criteria:**
- [ ] `GET /workspaces/:id/reports/balance-sheet?as_of=DATE` returns balance sheet as-of a date (defaults to today)
- [ ] `GET /workspaces/:id/reports/income-statement?start=DATE&end=DATE` returns income statement for a date range
- [ ] Both reports return accounts as hierarchical trees with rolled-up balances
**Dependencies:** R1, R2, cavekit-ledger.md R5, R6

### R8: Baseline Endpoints
**Description:** Endpoints to manage baselines and retrieve variance summaries.
**Acceptance Criteria:**
- [ ] `PUT /workspaces/:id/baselines` upserts a baseline for an account and period (editor+)
- [ ] `GET /workspaces/:id/baselines` returns all baselines, filterable by period
- [ ] `GET /workspaces/:id/summary/:periodType/:periodStart` returns a period summary with actuals, baselines, and variance per account
**Dependencies:** R1, R2, cavekit-baselines.md

## Out of Scope
- GraphQL or WebSocket APIs (future)
- Projection or retirement planning endpoints (future)
- File upload endpoints (future)
- Webhook or notification delivery endpoints (future)
- Public/read-only share links (future)

## Cross-References
- Depends on: cavekit-workspace.md — workspace and membership requirements
- Depends on: cavekit-ledger.md — account, transaction, posting requirements
- Depends on: cavekit-baselines.md — baseline and variance requirements
