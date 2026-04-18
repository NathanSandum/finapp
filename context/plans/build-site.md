# Build Site — MVP

Stack: Axum, sqlx, SQLite, Tauri, SvelteKit.
Workflow: TDD (failing test first, then implementation), PR per wave.

---

## Wave 1 — Foundation

*No dependencies. Establishes the project skeleton everything else builds on.*

### T1.1 — Database & Migration Infrastructure
**Kit:** —
**Tasks:**
- Add `sqlx` with SQLite feature to `backend/Cargo.toml`
- Add `sqlx-cli` to dev tooling, document `sqlx migrate run` in README
- Create `backend/migrations/` directory
- Write initial migration: `users`, `workspaces` placeholder tables (empty — schema filled per domain wave)
- Write a test that runs all migrations against an in-memory SQLite database and asserts they succeed

### T1.2 — Error Types
**Kit:** cavekit-api.md R3
**Tasks:**
- Define a top-level `AppError` enum in `backend/src/error.rs` covering: `NotFound`, `Forbidden`, `Unauthorized`, `Conflict`, `Validation(Vec<FieldError>)`, `Internal`
- Implement `IntoResponse` for `AppError` producing `{ code, message }` JSON (and `{ code, message, fields }` for validation errors)
- Write unit tests asserting each variant serialises to the correct HTTP status and JSON shape

### T1.3 — Test Utilities
**Kit:** —
**Tasks:**
- Write a `backend/src/test_helpers.rs` module providing: in-memory SQLite pool setup, migration runner, and a function that returns a fresh `sqlx::Pool<Sqlite>` per test
- Verify the helper compiles and the pool can execute a trivial query

---

## Wave 2 — Workspace Domain

*Depends on Wave 1. Tasks within this wave are independent.*

### T2.1 — User Model
**Kit:** cavekit-workspace.md R1
**Tasks:**
- Write failing tests: create user with unique email succeeds; duplicate email fails; password is not stored in plaintext
- Create migration: `users` table (`id`, `email`, `display_name`, `password_hash`, `created_at`, `updated_at`)
- Implement `User` struct and `UserRepository` with `create` and `find_by_email`
- Make tests pass

### T2.2 — Workspace Model
**Kit:** cavekit-workspace.md R2
**Tasks:**
- Write failing tests: create workspace succeeds; workspace has creator as owner; user can belong to multiple workspaces
- Create migration: `workspaces` table (`id`, `name`, `created_at`, `updated_at`)
- Implement `Workspace` struct and `WorkspaceRepository` with `create` and `find_by_id`
- Make tests pass

### T2.3 — Membership Events Log & Projection
**Kit:** cavekit-workspace.md R3, R6
**Tasks:**
- Write failing tests:
  - Appending an `owner_assigned` event creates a matching row in the projection
  - Removing the last owner is rejected before any event is written
  - The check-and-write is atomic (simulate concurrent removal; projection must never show zero owners)
  - A `viewer` role is correctly reflected in the projection
- Create migration: `membership_events` table (`id`, `workspace_id`, `user_id`, `actor_id`, `event_type`, `role`, `created_at`) and `workspace_members` projection table (`workspace_id`, `user_id`, `role`, `joined_at`) with unique index on `(workspace_id, user_id)`
- Implement `MembershipEvent` enum: `OwnerAssigned`, `Invited`, `InvitationAccepted`, `RoleChanged`, `Removed`, `Left`
- Implement `MembershipRepository`: `append_event` (writes event + updates projection in one transaction), `current_members`, `role_for_user`
- Implement last-owner guard in `append_event` for `Removed` and `Left` events
- Make tests pass

### T2.4 — Invitations
**Kit:** cavekit-workspace.md R4
**Tasks:**
- Write failing tests: valid invitation can be accepted; expired invitation cannot; already-member cannot be re-invited; acceptance appends `InvitationAccepted` event
- Create migration: `invitations` table (`id`, `workspace_id`, `invited_email`, `role`, `token`, `expires_at`, `accepted_at`, `created_by`)
- Implement `InvitationRepository` with `create`, `find_by_token`, `accept`
- Make tests pass

### T2.5 — Workspace Isolation
**Kit:** cavekit-workspace.md R5
**Tasks:**
- Write failing tests: queries for workspace A data return nothing when called with workspace B's ID; non-member receives no data
- Implement `is_member` check on `MembershipRepository`
- All repository methods that are workspace-scoped accept a `workspace_id` parameter and filter by it — verify in tests
- Make tests pass

---

## Wave 3 — Ledger Domain

*Depends on Wave 1. Tasks within this wave are independent.*

### T3.1 — Categories
**Kit:** cavekit-ledger.md R2
**Tasks:**
- Write failing tests: create category succeeds; duplicate name in same workspace fails; default categories created on workspace creation; soft-delete prevents deletion of categories with active line items
- Create migration: `categories` table (`id`, `workspace_id`, `name`, `description`, `deleted_at`, `created_at`, `updated_at`)
- Implement `Category` struct and `CategoryRepository` with `create`, `find_all`, `find_by_id`, `soft_delete`
- Implement default category seeding function (called after workspace creation)
- Make tests pass

### T3.2 — Accounts
**Kit:** cavekit-ledger.md R1
**Tasks:**
- Write failing tests: create account succeeds; duplicate name in same workspace fails; account types are validated; computed balance returns sum of cleared transactions; balance as-of a date works correctly
- Create migration: `accounts` table (`id`, `workspace_id`, `name`, `type`, `currency`, `deleted_at`, `created_at`, `updated_at`)
- Implement `Account` struct and `AccountRepository` with `create`, `find_all`, `find_by_id`, `soft_delete`, `balance`, `balance_as_of`
- Make tests pass

### T3.3 — Transactions & Line Items
**Kit:** cavekit-ledger.md R3, R4, R5
**Tasks:**
- Write failing tests:
  - Create transaction with line items succeeds
  - Line item amounts must sum to transaction total — violation rejected
  - Transaction with zero line items rejected
  - Transfer transaction references two accounts in same workspace
  - Soft-deleted transaction excluded from balance computation
  - Line item deletion rejected if it would leave transaction with zero items
- Create migrations: `transactions` table (`id`, `workspace_id`, `account_id`, `transfer_pair_id`, `date`, `description`, `total_amount`, `cleared`, `deleted_at`, `created_at`, `updated_at`); `line_items` table (`id`, `transaction_id`, `category_id`, `amount`, `note`, `deleted_at`, `created_at`, `updated_at`)
- Implement `Transaction`, `LineItem` structs and `TransactionRepository` with `create` (accepts line items atomically), `find_by_id`, `find_all` (with filters: date range, account, category, cleared, max 1000 rows), `soft_delete`; `LineItemRepository` with `add`, `update`, `soft_delete`
- Make tests pass

---

## Wave 4 — Baselines Domain

*Depends on Wave 3 (uses categories and transaction queries).*

### T4.1 — Baseline Model
**Kit:** cavekit-baselines.md R1
**Tasks:**
- Write failing tests: create baseline succeeds; duplicate category+period fails; zero amount baseline is valid; updating baseline does not affect past variance
- Create migration: `baselines` table (`id`, `workspace_id`, `category_id`, `period_type`, `period_start`, `amount`, `created_at`, `updated_at`) with unique index on `(workspace_id, category_id, period_start)`
- Implement `Baseline` struct and `BaselineRepository` with `upsert`, `find_all`, `find_by_period`
- Make tests pass

### T4.2 — Variance & Period Summary
**Kit:** cavekit-baselines.md R2, R3
**Tasks:**
- Write failing tests:
  - Variance = actual - baseline for matched category+period
  - Category with no baseline returns null variance, not zero
  - Category with baseline but no actuals returns negative variance equal to baseline
  - Period summary includes all categories with activity or a baseline
  - Variance is presented as absolute and percentage
- Implement `VarianceService`: `compute_variance`, `period_summary` — reads from `TransactionRepository` and `BaselineRepository`, no stored state
- Make tests pass

---

## Wave 5 — Auth & Middleware

*Depends on Wave 2 (user model).*

### T5.1 — Session Tokens
**Kit:** cavekit-api.md R1
**Tasks:**
- Write failing tests: login with valid credentials returns token; invalid credentials return 401; expired token returns 401; logout invalidates token
- Create migration: `sessions` table (`id`, `user_id`, `token_hash`, `expires_at`, `created_at`)
- Implement token generation (random, hashed at rest), `SessionRepository` with `create`, `validate`, `invalidate`
- Add `POST /auth/register` and `POST /auth/login` and `POST /auth/logout` handlers
- Make tests pass

### T5.2 — Auth & Role Middleware
**Kit:** cavekit-api.md R1, R2
**Tasks:**
- Write failing tests: missing token → 401; invalid token → 401; valid token but non-member → 403; viewer on write route → 403; editor on write route → 200
- Implement Axum extractor `AuthUser` that validates token and attaches user to request
- Implement Axum extractor `WorkspaceMember(role)` that checks membership and enforces minimum role
- Make tests pass

---

## Wave 6 — Workspace API

*Depends on Wave 5.*

### T6.1 — Workspace & Membership Endpoints
**Kit:** cavekit-api.md R4
**Tasks:**
- Write failing integration tests for each endpoint (use test HTTP client against real in-memory DB):
  - `POST /workspaces` → 201 with workspace + creator as owner
  - `GET /workspaces` → lists only caller's workspaces
  - `GET /workspaces/:id` → 200 for member, 403 for non-member
  - `POST /workspaces/:id/invitations` → 201 for owner, 403 for editor/viewer
  - `POST /invitations/:token/accept` → 200 on valid, 410 on expired
  - `DELETE /workspaces/:id/members/:userId` → 200 for owner, 403 for others, 409 when removing last owner
- Implement handlers wiring to Wave 2 repositories
- Make tests pass

---

## Wave 7 — Ledger API

*Depends on Wave 5, Wave 3.*

### T7.1 — Account Endpoints
**Kit:** cavekit-api.md R5
**Tasks:**
- Write failing integration tests for all account endpoints including balance in response
- Implement handlers wiring to Wave 3 repositories
- Make tests pass

### T7.2 — Category Endpoints
**Kit:** cavekit-api.md R7
**Tasks:**
- Write failing integration tests including 409 on delete with active line items
- Implement handlers
- Make tests pass

### T7.3 — Transaction & Line Item Endpoints
**Kit:** cavekit-api.md R6
**Tasks:**
- Write failing integration tests for all transaction and line item endpoints including sum enforcement and soft-delete behaviour
- Implement handlers
- Make tests pass

---

## Wave 8 — Baselines API

*Depends on Wave 7, Wave 4.*

### T8.1 — Baseline & Summary Endpoints
**Kit:** cavekit-api.md R8
**Tasks:**
- Write failing integration tests: upsert creates or updates; period summary returns correct variance; categories with no activity appear if baseline exists
- Implement handlers wiring to Wave 4 services
- Make tests pass

---

## Wave 9 — Frontend

*Depends on Wave 6, Wave 7, Wave 8. SvelteKit + Tauri.*

### T9.1 — Auth Pages
**Tasks:**
- Register page (email, display name, password)
- Login page
- Session persistence (store token, redirect on expiry)
- Tests: form validation, redirect on success

### T9.2 — Workspace Shell
**Tasks:**
- Workspace list / create page
- Workspace switcher in nav
- Invite member flow (owner only)
- Member list with role badges

### T9.3 — Accounts View
**Tasks:**
- Account list with current balances
- Create / edit account form
- Soft-delete with confirmation

### T9.4 — Transaction Entry
**Tasks:**
- Transaction list with filters (date range, account, category, cleared)
- Create transaction form with dynamic line item rows (add/remove, running total validation)
- Edit transaction
- Mark cleared / pending
- Transfer form (selects two accounts)

### T9.5 — Period Summary View
**Tasks:**
- Period picker (monthly / quarterly)
- Summary table: category, actual, baseline, variance (absolute + %)
- Baseline inline editing (upsert on blur)
- Visual distinction between positive and negative variance (informational, not judgemental)
