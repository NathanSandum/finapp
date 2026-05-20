# API Routes

Base: all routes registered in `lib.rs:build_router()`.
Auth: Bearer token in `Authorization` header. Extractors handle validation.

## Response envelope (JSend)

Every response uses the [JSend](https://github.com/omniti-labs/jsend) envelope:

```json
// success — request succeeded; payload in `data` (may be null)
{ "status": "success", "data": <payload> }

// fail — client-side rejection (4xx). For validation errors, `data` is a
// field→message map; for other client errors, `data` carries `code` + `message`.
{ "status": "fail", "data": { "code": "not_found", "message": "..." } }
{ "status": "fail", "data": { "<field>": "<message>", ... } }   // 422

// error — server-side failure (5xx). `code` and `data` optional.
{ "status": "error", "message": "an unexpected error occurred" }
```

The shapes documented under each route below describe the **`data`** payload only.

## Auth Routes (no token required)

### POST /auth/register
```json
// Request
{ "email": "str", "display_name": "str", "password": "str" }
// data on 200
{ "user": { "id": "uuid", "email": "str", "display_name": "str" }, "token": "uuid", "expires_at": "timestamp" }
// 409: duplicate email
```

### POST /auth/login
```json
// Request
{ "email": "str", "password": "str" }
// data on 200: same shape as register
// 401: wrong credentials
```

### POST /auth/logout
Requires token. Deletes session. `data` is `null`.

## Workspace Routes (token required)

### POST /workspaces
```json
// Request
{ "name": "str" }
// data on 201
{ "workspace": "uuid", "name": "str", "created_at": "timestamp", "updated_at": "timestamp" }
```
Creates workspace + owner membership + default account tree (18 accounts).

### GET /workspaces
```json
// data on 200
{ "workspaces": [{ "workspace": "uuid", "name": "str", ... }] }
```
Returns only workspaces where authenticated user is a member.

### GET /workspaces/{workspace}
```json
// data on 200
{ "workspace": "uuid", "name": "str", "created_at": "str", "your_role": "owner|editor|viewer", "members": [...] }
// 403: not a member
```

### DELETE /workspaces/{workspace}/members/{user}
Permission checked via `permissions.rs` (Action::Delete × Resource). Self-removal allowed for any role. Owner removal of non-owner members allowed. Last owner removal blocked (409 Conflict). `data` is `null` on success.

## Account Routes (token + workspace membership required)

### GET /workspaces/{workspace}/accounts
```json
// data on 200
{ "accounts": [{ "id": "uuid", "parent_id": "uuid|null", "name": "str", "account_type": "asset|...", "currency": "CAD", "role": "assets_root|liabilities_root|equity_root|revenue_root|expenses_root|cash|opening_equity|null" }] }
```
Returns flat list of all active accounts. Filtered by workspace membership. `role` is non-null only on anchor accounts (5 roots, Cash = Chequing, OpeningEquity = Opening Balances).

### POST /workspaces/{workspace}/accounts
Requires Editor role. Root accounts are seeded automatically and cannot be created via API.
```json
// Child account (inherits type from parent)
{ "name": "str", "parent_id": "uuid" }
// Optional: "currency": "USD" (default "CAD")

// data on 201
{ "id": "uuid", "parent_id": "uuid|null", "name": "str", "account_type": "str", "currency": "str", "role": null }
// 422: parent_id missing
// 409: duplicate name under same parent
```

### DELETE /workspaces/{workspace}/accounts/{account}
Requires Editor role. `data` is `null`.
409 if account has active children, active postings, or is an anchor (role is non-null).

### GET /workspaces/{workspace}/accounts/{account}/balance
```json
// data on 200
{ "account": "uuid", "balance": 1200 }
```
Returns sum of postings on non-deleted entries for this account.

### GET /workspaces/{workspace}/anchors
```json
// data on 200
{
  "assets":         { ...account, "role": "assets_root" },
  "liabilities":    { ...account, "role": "liabilities_root" },
  "equity":         { ...account, "role": "equity_root" },
  "revenue":        { ...account, "role": "revenue_root" },
  "expenses":       { ...account, "role": "expenses_root" },
  "cash":           { ...account, "role": "cash" },
  "opening_equity": { ...account, "role": "opening_equity" }
}
```
Returns the 7 anchor accounts the workspace guarantees. Frontend can rely on these for statement assembly without name-matching.

## Entry Routes (token + workspace membership required)

### POST /workspaces/{workspace}/entries
Requires Editor role.
```json
// Request
{
  "date": "YYYY-MM-DD",
  "description": "str",
  "postings": [
    { "account_id": "uuid", "amount": -1200, "note": "optional" },
    { "account_id": "uuid", "amount": 1200 }
  ]
}
// data on 201
{ "id": "uuid", "date": "str", "description": "str", "postings": [...] }
// 422: non-zero sum, fewer than 2 postings, zero amount
```

### GET /workspaces/{workspace}/entries
```json
// data on 200
{ "entries": [{ "id": "uuid", "date": "str", "description": "str", "postings": [...] }] }
```
Returns all non-deleted entries with their postings. Ordered by date descending.

### GET /workspaces/{workspace}/entries/{entry}
```json
// data on 200
{ "id": "uuid", "date": "str", "description": "str", "postings": [{ "id": "uuid", "account_id": "uuid", "amount": int, "note": "str" }] }
// 404: entry not found or wrong workspace
```

### DELETE /workspaces/{workspace}/entries/{entry}
Requires Editor role. Soft-deletes entry. `data` is `null`.

## Report Routes (token + workspace membership required)

### GET /workspaces/{workspace}/reports/balance-sheet?as_of=YYYY-MM-DD
```json
// data on 200
{ "as_of_date": "str", "assets": [...], "liabilities": [...], "equity": [...], "total_assets": int, "total_liabilities": int, "total_equity": int, "net_income": int, "total_liabilities_and_equity": int }
// 422: missing as_of parameter
```
Per-account detail arrays contain `{ "account": "uuid", "name": "str", "balance": int }`.

### GET /workspaces/{workspace}/reports/income-statement?start_date=...&end_date=...
```json
// data on 200
{ "start_date": "str", "end_date": "str", "revenue": [...], "expenses": [...], "total_revenue": int, "total_expenses": int, "net_income": int }
// 422: missing start_date or end_date
```

## Baseline Routes (token + workspace membership required)

### POST /workspaces/{workspace}/baselines
Requires Editor role.
```json
// Request
{ "account_id": "uuid", "period_start": "YYYY-MM-DD", "amount": 50000 }
// data on 201: full baseline object
// 409: duplicate (same account + period_start)
```

### GET /workspaces/{workspace}/baselines
```json
// data on 200
{ "baselines": [{ "id": "uuid", "account": "uuid", "period_type": "monthly", "period_start": "str", "amount": int, ... }] }
```

### DELETE /workspaces/{workspace}/baselines/{baseline}
Requires Editor role. Hard-deletes baseline. `data` is `null`.

### GET /workspaces/{workspace}/baselines/{baseline}/variance
```json
// data on 200
{ "baseline": "uuid", "amount": 50000, "actual": 7500, "variance": -42500 }
```
Positive variance = over budget. Actual includes descendant account postings within period.

## Recurring Rule Routes (token + workspace membership required)

### POST /workspaces/{workspace}/recurring
Requires Editor role. The request specifies the schedule by tag — the backend owns the tag → RRULE mapping. Allowed tags: `weekly`, `biweekly`, `monthly-first`, `monthly-last`, `yearly`. Unknown tags are rejected (400/422).
```json
// Request
{
  "description": "Monthly rent",
  "schedule": "monthly-first",
  "dtstart": "2026-01-01",
  "templates": [
    { "account_id": "uuid", "amount": -150000, "note": "optional" },
    { "account_id": "uuid", "amount": 150000 }
  ]
}
// data on 201
{
  "id": "uuid", "workspace": "uuid", "description": "str",
  "schedule": "monthly-first" | null,
  "rrule": "str", "dtstart": "YYYY-MM-DD", "last_fired_on": null,
  "deleted_at": null, "created_at": "...", "updated_at": "...",
  "templates": [{ "id": "uuid", "rule": "uuid", "account": "uuid", "amount": int, "note": "str", ... }],
  "next_pending": "YYYY-MM-DD" | null
}
// 400/422: unknown schedule tag, fewer than 2 templates, non-zero sum, zero amount
```
The `rrule` field is still echoed in the response (RRULE remains the source of truth in storage), but clients should display the `schedule` tag — `rrule` may be `null` in the response when a rule was stored with an RRULE that doesn't map to any known tag.

### GET /workspaces/{workspace}/recurring
```json
// data on 200
{ "recurring_rules": [ { ...rule, "templates": [...], "next_pending": "..." }, ... ] }
```
Active (non-deleted) rules, ordered by `dtstart` ascending.

### GET /workspaces/{workspace}/recurring/{rule}
```json
// data on 200: same shape as a single entry from list
// 404: rule not found, soft-deleted, or wrong workspace
```

### DELETE /workspaces/{workspace}/recurring/{rule}
Requires Editor role. Soft-deletes the rule. `data` is `null`.

### POST /workspaces/{workspace}/recurring/runs
Requires Editor role. Creates a "run": iterates every active rule and posts any due occurrences as real entries (one entry per occurrence, with `source_rule_id` set to the rule). Idempotent — calling it twice in a row posts zero the second time. Intended to be called by the frontend on workspace load.
```json
// data on 201
{ "posted": int, "at": "timestamp" }
```

## Entry source_rule

Entries now carry a `source_rule` field (UUID of the recurring rule that generated them, or null for manually-created entries). Both `GET /entries` and `GET /entries/{id}` include it in their response payloads.
