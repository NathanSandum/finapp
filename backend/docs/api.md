# API Routes

Base: all routes registered in `lib.rs:build_router()`.
Auth: Bearer token in `Authorization` header. Extractors handle validation.

## Auth Routes (no token required)

### POST /auth/register
```json
// Request
{ "email": "str", "display_name": "str", "password": "str" }
// Response 200
{ "user": { "id": "uuid", "email": "str", "display_name": "str" }, "token": "uuid", "expires_at": "timestamp" }
// Error 409: duplicate email
```

### POST /auth/login
```json
// Request
{ "email": "str", "password": "str" }
// Response 200: same shape as register
// Error 401: wrong credentials
```

### POST /auth/logout
Requires token. Deletes session. Returns `{ "status": "ok" }`.

## Workspace Routes (token required)

### POST /workspaces
```json
// Request
{ "name": "str" }
// Response 201
{ "workspace": "uuid", "name": "str", "created_at": "timestamp", "updated_at": "timestamp" }
```
Creates workspace + owner membership + default account tree (18 accounts).

### GET /workspaces
```json
// Response 200
{ "workspaces": [{ "workspace": "uuid", "name": "str", ... }] }
```
Returns only workspaces where authenticated user is a member.

### GET /workspaces/{workspace}
```json
// Response 200
{ "workspace": "uuid", "name": "str", "created_at": "str", "your_role": "owner|editor|viewer", "members": [...] }
// Error 403: not a member
```

### DELETE /workspaces/{workspace}/members/{user}
Permission checked via `permissions.rs` (Action::Delete × Resource). Self-removal allowed for any role. Owner removal of non-owner members allowed. Last owner removal blocked (409 Conflict).

## Account Routes (token + workspace membership required)

### GET /workspaces/{workspace}/accounts
```json
// Response 200
{ "accounts": [{ "id": "uuid", "parent_id": "uuid|null", "name": "str", "account_type": "asset|...", "currency": "CAD" }] }
```
Returns flat list of all active accounts. Filtered by workspace membership.

### POST /workspaces/{workspace}/accounts
Requires Editor role.
```json
// Child account (inherits type from parent)
{ "name": "str", "parent_id": "uuid" }
// Root account (type required)
{ "name": "str", "account_type": "asset|liability|equity|revenue|expense" }
// Optional: "currency": "USD" (default "CAD")

// Response 201
{ "id": "uuid", "parent_id": "uuid|null", "name": "str", "account_type": "str", "currency": "str" }
// Error 422: root account without account_type
// Error 409: duplicate name under same parent
```

### DELETE /workspaces/{workspace}/accounts/{account}
Requires Editor role. Returns `{ "status": "ok" }`.
Error 409 if account has active children or active postings.

### GET /workspaces/{workspace}/accounts/{account}/balance
```json
// Response 200
{ "account": "uuid", "cleared_balance": 1200 }
```
Returns sum of postings on cleared, non-deleted entries for this account.
