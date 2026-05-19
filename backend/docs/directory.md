# Backend Directory

```
backend/
  Cargo.toml                        # Dependencies: axum, diesel, sqlite, argon2, chrono, uuid
  src/
    main.rs                         # Entry point — builds pool, runs axum server
    lib.rs                          # build_router() — all route registration, DbPool type alias
    schema.rs                       # Diesel table! macros — source of truth for DB shape
    error.rs                        # AppError enum → HTTP status mapping, FieldError for validation
    auth.rs                         # Axum extractors: AuthUser (token→user), WorkspaceMember (token→user+workspace+role)
    permissions.rs                  # Role-based permission checks (Action × Resource matrix)
    test_helpers.rs                 # setup_test_db() — in-memory SQLite + migrations, MIGRATIONS constant
    domain.rs                       # Module declarations: account, baseline, entry, recurring, report, session, user, workspace
    routes.rs                       # Module declarations: accounts, auth, baselines, entries, recurring, reports, workspaces
    domain/
      user.rs                       # User, UserRef, NewUser (argon2 hashing), email lookup, password verify
      session.rs                    # Session (SHA-256 token hash), NewSession, validate/invalidate, 1-week expiry
      workspace.rs                  # Workspace, WorkspaceRef, NewWorkspace (creates membership + default accounts)
                                    # WorkspaceUser, MembershipEvent (event-sourced membership)
                                    # Role enum (Viewer < Editor < Owner) with FromSql/ToSql
      account.rs                    # Account, AccountRef, AccountType enum (Asset/Liability/Equity/Revenue/Expense)
                                    # NewAccount (root or child builder), create_default_tree()
                                    # Hierarchy: full_name(), ancestors(), children(), descendants(), soft_delete()
                                    # Balance: balance(), balance_with_descendants(), balance_as_of(),
                                    #   balance_as_of_with_descendants(), balance_between(), balance_between_with_descendants()
      entry.rs                      # EntryRef, PostingRef, Entry, Posting, NewEntry builder
                                    # Zero-sum validation, PostingInput, query methods on owning types
                                    # AccountRef::entries(), Account::entries_with_descendants(), WorkspaceRef::entries()
      report.rs                     # BalanceSheet (as_of), IncomeStatement (for_period), AccountBalance
                                    # Reports aggregate account balances with per-account detail + totals
      baseline.rs                   # BaselineRef, PeriodType (Monthly), Baseline, NewBaseline builder
                                    # Baseline::variance() computes actual - budget for period
                                    # AccountRef::baselines(), WorkspaceRef::baselines()
      recurring.rs                  # RecurringRuleRef, RecurringRule, RecurringPostingTemplate
                                    # Schedule newtype (parsed+validated RRULE + dtstart), next_after()
                                    # PostingTemplates newtype (≥2, non-zero, zero-sum invariant)
                                    # NewRecurringRule builder, RecurringRule::next_pending()
                                    # WorkspaceRef::recurring_rules()
    routes/
      auth.rs                       # POST /auth/register, /auth/login, /auth/logout
      workspaces.rs                 # POST/GET /workspaces, GET /workspaces/:id, DELETE .../members/:user
      accounts.rs                   # GET/POST /workspaces/:id/accounts, DELETE .../accounts/:id, GET .../balance
      entries.rs                    # POST/GET /workspaces/:id/entries, GET/DELETE .../entries/:id
      reports.rs                    # GET .../reports/balance-sheet, GET .../reports/income-statement
      baselines.rs                  # POST/GET /workspaces/:id/baselines, DELETE .../baselines/:id, GET .../variance
      recurring.rs                  # POST/GET /workspaces/:id/recurring, GET/DELETE .../recurring/:id
                                    # Response embeds templates and computed next_pending
  migrations/
    00000000000001_initial/         # Empty placeholder
    00000000000002_users/           # users table
    00000000000003_workspaces/      # workspaces table
    00000000000004_membership/      # workspace_members + membership_events tables
    00000000000005_categories/      # categories table (DROPPED in migration 10)
    00000000000006_accounts/        # accounts table (RECREATED in migration 10)
    00000000000007_transactions/    # transactions table (RENAMED to entries in migration 12)
    00000000000008_baselines/       # baselines table (DROPPED in migration 10, RECREATED in migration 13)
    00000000000009_sessions/        # sessions table
    00000000000010_double_entry/    # Double-entry rewrite: drop categories/line_items/baselines,
                                    # recreate accounts (hierarchical) + transactions (slim)
    00000000000011_postings/        # Postings table (entry_id, account_id, amount, note) + indexes
    00000000000012_rename_.../      # Rename transactions→entries, transaction_id→entry_id
    00000000000013_baselines/       # Baselines table (account_id, period_type, period_start, amount)
    00000000000018_recurring/       # recurring_rules + recurring_posting_templates tables
  tests/
    api.rs                          # Integration tests — in-memory DB, axum oneshot requests
                                    # Covers: auth, workspaces, accounts, entries, reports, baselines, recurring
```
