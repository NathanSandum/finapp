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

## Entry Routes (token + workspace membership required)

### POST /workspaces/{workspace}/entries
Requires Editor role.
```json
// Request
{
  "date": "YYYY-MM-DD",
  "description": "str",
  "cleared": false,
  "postings": [
    { "account_id": "uuid", "amount": -1200, "note": "optional" },
    { "account_id": "uuid", "amount": 1200 }
  ]
}
// Response 201
{ "id": "uuid", "date": "str", "description": "str", "cleared": false, "postings": [...] }
// Error 422: non-zero sum, fewer than 2 postings, zero amount
```

### GET /workspaces/{workspace}/entries
```json
// Response 200
{ "entries": [{ "id": "uuid", "date": "str", "description": "str", "cleared": bool, "postings": [...] }] }
```
Returns all non-deleted entries with their postings. Ordered by date descending.

### GET /workspaces/{workspace}/entries/{entry}
```json
// Response 200
{ "id": "uuid", "date": "str", "description": "str", "cleared": bool, "postings": [{ "id": "uuid", "account_id": "uuid", "amount": int, "note": "str" }] }
// Error 404: entry not found or wrong workspace
```

### DELETE /workspaces/{workspace}/entries/{entry}
Requires Editor role. Soft-deletes entry. Returns `{ "status": "ok" }`.

## Report Routes (token + workspace membership required)

### GET /workspaces/{workspace}/reports/balance-sheet?as_of=YYYY-MM-DD
```json
// Response 200
{ "as_of_date": "str", "assets": [...], "liabilities": [...], "equity": [...], "total_assets": int, "total_liabilities": int, "total_equity": int, "net_income": int, "total_liabilities_and_equity": int }
// Error 422: missing as_of parameter
```
Per-account detail arrays contain `{ "account": "uuid", "name": "str", "balance": int }`.

### GET /workspaces/{workspace}/reports/income-statement?start_date=...&end_date=...
```json
// Response 200
{ "start_date": "str", "end_date": "str", "revenue": [...], "expenses": [...], "total_revenue": int, "total_expenses": int, "net_income": int }
// Error 422: missing start_date or end_date
```

## Baseline Routes (token + workspace membership required)

### POST /workspaces/{workspace}/baselines
Requires Editor role.
```json
// Request
{ "account_id": "uuid", "period_start": "YYYY-MM-DD", "amount": 50000 }
// Response 201: full baseline object
// Error 409: duplicate (same account + period_start)
```

### GET /workspaces/{workspace}/baselines
```json
// Response 200
{ "baselines": [{ "id": "uuid", "account": "uuid", "period_type": "monthly", "period_start": "str", "amount": int, ... }] }
```

### DELETE /workspaces/{workspace}/baselines/{baseline}
Requires Editor role. Hard-deletes baseline. Returns `{ "status": "ok" }`.

### GET /workspaces/{workspace}/baselines/{baseline}/variance
```json
// Response 200
{ "baseline": "uuid", "amount": 50000, "actual": 7500, "variance": -42500 }
```
Positive variance = over budget. Actual includes descendant account postings within period.
