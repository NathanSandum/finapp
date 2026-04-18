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
**Description:** Endpoints to manage accounts within a workspace.
**Acceptance Criteria:**
- [ ] `POST /workspaces/:id/accounts` creates an account (editor+)
- [ ] `GET /workspaces/:id/accounts` returns all active accounts with current balances
- [ ] `GET /workspaces/:id/accounts/:accountId` returns account detail and current balance
- [ ] `PATCH /workspaces/:id/accounts/:accountId` updates account name or type (editor+)
- [ ] `DELETE /workspaces/:id/accounts/:accountId` soft-deletes an account (editor+)
**Dependencies:** R1, R2, cavekit-ledger.md R1, R5

### R6: Transaction Endpoints
**Description:** Endpoints to create, read, update, and delete transactions and their line items.
**Acceptance Criteria:**
- [ ] `POST /workspaces/:id/transactions` creates a transaction with line items (editor+)
- [ ] `GET /workspaces/:id/transactions` returns transactions with filter support (date range, account, category, cleared status), capped at 1000 rows, ordered by date descending
- [ ] `GET /workspaces/:id/transactions/:txId` returns a transaction with all line items
- [ ] `PATCH /workspaces/:id/transactions/:txId` updates transaction fields (editor+)
- [ ] `DELETE /workspaces/:id/transactions/:txId` soft-deletes a transaction (editor+)
- [ ] `POST /workspaces/:id/transactions/:txId/line-items` adds a line item (editor+)
- [ ] `PATCH /workspaces/:id/transactions/:txId/line-items/:itemId` updates a line item (editor+)
- [ ] `DELETE /workspaces/:id/transactions/:txId/line-items/:itemId` removes a line item, provided at least one remains (editor+)
**Dependencies:** R1, R2, cavekit-ledger.md R3, R4, R6, R7

### R7: Category Endpoints
**Description:** Endpoints to manage categories within a workspace.
**Acceptance Criteria:**
- [ ] `POST /workspaces/:id/categories` creates a category (editor+)
- [ ] `GET /workspaces/:id/categories` returns all active categories
- [ ] `PATCH /workspaces/:id/categories/:catId` updates a category name or description (editor+)
- [ ] `DELETE /workspaces/:id/categories/:catId` soft-deletes a category with no active line items, or returns a 409 if line items exist (editor+)
**Dependencies:** R1, R2, cavekit-ledger.md R2, R7

### R8: Baseline Endpoints
**Description:** Endpoints to manage baselines and retrieve variance summaries.
**Acceptance Criteria:**
- [ ] `PUT /workspaces/:id/baselines` upserts a baseline for a category and period (editor+)
- [ ] `GET /workspaces/:id/baselines` returns all baselines, filterable by period
- [ ] `GET /workspaces/:id/summary/:periodType/:periodStart` returns a period summary with actuals, baselines, and variance per category
**Dependencies:** R1, R2, cavekit-baselines.md

## Out of Scope
- GraphQL or WebSocket APIs (future)
- Projection or retirement planning endpoints (future)
- File upload endpoints (future)
- Webhook or notification delivery endpoints (future)
- Public/read-only share links (future)

## Cross-References
- Depends on: cavekit-workspace.md — workspace and membership requirements
- Depends on: cavekit-ledger.md — account, transaction, line item, category requirements
- Depends on: cavekit-baselines.md — baseline and variance requirements
