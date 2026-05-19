# Cavekit: Recurring

## Scope

Covers recurring rules that produce entries on a schedule. A rule is a workspace-scoped template — description, schedule, and a set of posting templates that mirror the manual entry shape (2+ postings summing to zero). Schedules use RFC 5545 RRULE syntax. This MVP defines storage, validation, and CRUD; materialization (auto-posting due rules) is **not** in scope here and is deferred to a future kit.

## Requirements

### R1: Recurring Rule

**Description:** A recurring rule is a template that describes a repeating entry pattern. It captures *what* the entry looks like (postings) and *when* it should occur (schedule).

**Acceptance Criteria:**
- [ ] A rule has a unique identifier, workspace, description, RRULE string, dtstart date, and timestamps
- [ ] A rule belongs to exactly one workspace
- [ ] A rule has at least two posting templates whose amounts sum to zero
- [ ] A rule is soft-deletable (deleted_at) and excluded from default queries when deleted
- [ ] A rule retains a `last_fired_on` date (nullable) that records the most recent occurrence already materialized

**Dependencies:** cavekit-ledger.md R1, R2, R3, cavekit-workspace.md R2

### R2: Posting Templates

**Description:** Each rule owns a set of posting templates. A template mirrors a posting: account reference, amount in integer cents, optional note. Templates are created and destroyed atomically with their parent rule.

**Acceptance Criteria:**
- [ ] A template has a unique identifier, rule reference, account reference, integer-cent amount, optional note
- [ ] A template's amount is non-zero
- [ ] Templates have no independent CRUD; they are written and removed only via the rule
- [ ] Deleting a rule (soft) does not delete its templates from the database — they remain queryable for audit

**Dependencies:** R1

### R3: RRULE Schedule

**Description:** Schedules are stored as RFC 5545 RRULE strings (e.g. `FREQ=MONTHLY;BYMONTHDAY=1`, `FREQ=WEEKLY;INTERVAL=2;BYDAY=FR;COUNT=10`). The dtstart anchors the recurrence.

**Acceptance Criteria:**
- [ ] An invalid RRULE string is rejected at insert time with a validation error
- [ ] The rule supports FREQ values: DAILY, WEEKLY, MONTHLY, YEARLY
- [ ] The rule supports INTERVAL (every N units)
- [ ] The rule supports BYMONTHDAY (day-of-month anchored, including -1 for last day)
- [ ] The rule supports UNTIL and COUNT end conditions
- [ ] Given a rule and an "after" date, the next scheduled occurrence date can be computed deterministically
- [ ] An exhausted schedule (past UNTIL or all COUNT fired) returns no next occurrence

**Dependencies:** R1

### R4: Rule CRUD

**Description:** Recurring rules are created, listed, fetched, and soft-deleted through the API. Mutations require Editor role.

**Acceptance Criteria:**
- [ ] A workspace member with Editor role can create a rule with description, dtstart, RRULE, and posting templates
- [ ] Any workspace member can list active rules for the workspace
- [ ] Any workspace member can fetch a single rule (with templates) by id
- [ ] A workspace member with Editor role can soft-delete a rule
- [ ] Non-members are forbidden from all rule operations
- [ ] Validation errors (non-zero sum, fewer than 2 templates, zero amount, invalid RRULE) return 422 with field-level detail

**Dependencies:** R1, R2, R3, cavekit-workspace.md R5

## Out of Scope
- Auto-materialization of due rules into entries (future)
- Background job / cron daemon (future)
- Editing posting templates in place (delete + recreate the rule instead)
- EXDATE, RDATE, multiple RRULEs per rule, timezone handling (always local date)
- Linking materialized entries back to their source rule (future, requires materialization)

## Cross-References
- Depends on: cavekit-ledger.md R1, R2, R3 — templates reference accounts and mirror posting shape
- Depends on: cavekit-workspace.md R2, R5 — rules are workspace-scoped, role-gated
- Depended on by: cavekit-api.md — recurring endpoints derive from R4
