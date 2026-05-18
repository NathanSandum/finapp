use chrono::{NaiveDate, NaiveDateTime};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::{accounts, entries, postings};

// ── AccountType ──────────────────────────────────────────────────────────────

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
    diesel::expression::AsExpression, diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Asset => "asset",
            AccountType::Liability => "liability",
            AccountType::Equity => "equity",
            AccountType::Revenue => "revenue",
            AccountType::Expense => "expense",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "asset" => Some(AccountType::Asset),
            "liability" => Some(AccountType::Liability),
            "equity" => Some(AccountType::Equity),
            "revenue" => Some(AccountType::Revenue),
            "expense" => Some(AccountType::Expense),
            _ => None,
        }
    }
}

impl FromSql<Text, Sqlite> for AccountType {
    fn from_sql(
        bytes: <Sqlite as Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        AccountType::from_str(&s).ok_or_else(|| format!("unknown account type: {s}").into())
    }
}

impl ToSql<Text, Sqlite> for AccountType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(self.as_str(), out)
    }
}


// ── AccountRole ──────────────────────────────────────────────────────────────
//
// A small fixed set of well-known accounts the 3-statement model relies on.
// Each workspace has at most one active account per role (enforced by a
// partial unique index on (workspace_id, role) where role IS NOT NULL).
// Accounts without a role have role = None — that's the common case.

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
    diesel::expression::AsExpression, diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum AccountRole {
    AssetsRoot,
    LiabilitiesRoot,
    EquityRoot,
    RevenueRoot,
    ExpensesRoot,
    Cash,
    OpeningEquity,
}

impl AccountRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountRole::AssetsRoot => "assets_root",
            AccountRole::LiabilitiesRoot => "liabilities_root",
            AccountRole::EquityRoot => "equity_root",
            AccountRole::RevenueRoot => "revenue_root",
            AccountRole::ExpensesRoot => "expenses_root",
            AccountRole::Cash => "cash",
            AccountRole::OpeningEquity => "opening_equity",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "assets_root" => Some(AccountRole::AssetsRoot),
            "liabilities_root" => Some(AccountRole::LiabilitiesRoot),
            "equity_root" => Some(AccountRole::EquityRoot),
            "revenue_root" => Some(AccountRole::RevenueRoot),
            "expenses_root" => Some(AccountRole::ExpensesRoot),
            "cash" => Some(AccountRole::Cash),
            "opening_equity" => Some(AccountRole::OpeningEquity),
            _ => None,
        }
    }
}

impl FromSql<Text, Sqlite> for AccountRole {
    fn from_sql(
        bytes: <Sqlite as Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let raw = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        AccountRole::from_str(&raw).ok_or_else(|| format!("unknown account role: {raw}").into())
    }
}

impl ToSql<Text, Sqlite> for AccountRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(self.as_str(), out)
    }
}


// ── Models ────────────────────────────────────────────────────────────────────

#[derive(
    Hash, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType,
)]
#[serde(transparent)]
pub struct AccountRef(String);

impl std::fmt::Display for AccountRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Account {
    pub id: AccountRef,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    pub parent_id: Option<AccountRef>,
    pub name: String,
    pub account_type: AccountType,
    pub currency: String,
    pub role: Option<AccountRole>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Account {
    pub fn as_ref(&self) -> &AccountRef {
        &self.id
    }

    pub fn from_ref(
        account_ref: &AccountRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Account>, AppError> {
        accounts::table
            .find(account_ref)
            .select(Account::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("account lookup failed: {err}")))
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Returns the colon-delimited full path from root to this account.
    pub fn full_name(&self, conn: &mut SqliteConnection) -> Result<String, AppError> {
        let ancestors = self.ancestors(conn)?;
        let mut parts: Vec<&str> = ancestors.iter().map(|a| a.name.as_str()).collect();
        parts.push(&self.name);
        Ok(parts.join(":"))
    }

    /// Returns ancestors from root down to (but not including) self.
    pub fn ancestors(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        let mut result = Vec::new();
        let mut current_parent_id = self.parent_id.clone();

        while let Some(ref parent_id) = current_parent_id {
            let parent = Account::from_ref(parent_id, conn)?
                .ok_or_else(|| AppError::Internal("broken parent chain".to_string()))?;
            current_parent_id = parent.parent_id.clone();
            result.push(parent);
        }

        result.reverse();
        Ok(result)
    }

    /// Returns direct children of this account.
    pub fn children(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        accounts::table
            .filter(accounts::parent_id.eq(&self.id))
            .filter(accounts::deleted_at.is_null())
            .select(Account::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("children query failed: {err}")))
    }

    /// Returns all transitive descendants (BFS).
    pub fn descendants(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        let mut result = Vec::new();
        let mut queue = vec![self.id.clone()];

        while let Some(parent_id) = queue.pop() {
            let children: Vec<Account> = accounts::table
                .filter(accounts::parent_id.eq(&parent_id))
                .filter(accounts::deleted_at.is_null())
                .select(Account::as_select())
                .load(conn)
                .map_err(|err| AppError::Internal(format!("descendants query failed: {err}")))?;

            for child in children {
                queue.push(child.id.clone());
                result.push(child);
            }
        }

        Ok(result)
    }

    pub fn rename(
        &self,
        new_name: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Account, AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(accounts::table.find(&self.id))
            .set((
                accounts::name.eq(new_name),
                accounts::updated_at.eq(now),
            ))
            .execute(conn)?;

        Account::from_ref(&self.id, conn)?
            .ok_or_else(|| AppError::Internal("account disappeared after rename".to_string()))
    }

    pub fn reparent(
        &self,
        new_parent: &AccountRef,
        conn: &mut SqliteConnection,
    ) -> Result<Account, AppError> {
        if self.is_root() {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: "root accounts cannot be reparented".into(),
            }]));
        }

        if *new_parent == self.id {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: "cannot reparent account to itself".into(),
            }]));
        }

        let parent = Account::from_ref(new_parent, conn)?
            .ok_or(AppError::NotFound("parent account".into()))?;

        if parent.workspace != self.workspace {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: "new parent must be in the same workspace".into(),
            }]));
        }

        if parent.account_type != self.account_type {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: format!(
                    "cannot reparent {} account under {} account",
                    self.account_type.as_str(),
                    parent.account_type.as_str()
                ),
            }]));
        }

        let descendants = self.descendants(conn)?;
        if descendants.iter().any(|descendant| descendant.id == *new_parent) {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: "cannot reparent account under its own descendant".into(),
            }]));
        }

        let now = chrono::Utc::now().naive_utc();
        diesel::update(accounts::table.find(&self.id))
            .set((
                accounts::parent_id.eq(Some(new_parent)),
                accounts::updated_at.eq(now),
            ))
            .execute(conn)?;

        Account::from_ref(&self.id, conn)?
            .ok_or_else(|| AppError::Internal("account disappeared after reparent".to_string()))
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        // Anchor accounts are part of the workspace invariant and must always exist.
        if self.role.is_some() {
            return Err(AppError::Conflict(
                "cannot delete an anchor account".to_string(),
            ));
        }

        // Prevent deleting accounts with active children
        let active_children_count: i64 = accounts::table
            .filter(accounts::parent_id.eq(&self.id))
            .filter(accounts::deleted_at.is_null())
            .count()
            .get_result(conn)
            .map_err(|err| AppError::Internal(format!("children count failed: {err}")))?;

        if active_children_count > 0 {
            return Err(AppError::Conflict(
                "cannot delete account with active children".to_string(),
            ));
        }

        // Prevent deleting accounts with postings on active entries
        let active_posting_count: i64 = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq(&self.id))
            .filter(entries::deleted_at.is_null())
            .count()
            .get_result(conn)
            .map_err(|err| AppError::Internal(format!("posting count failed: {err}")))?;

        if active_posting_count > 0 {
            return Err(AppError::Conflict(
                "cannot delete account with active entries".to_string(),
            ));
        }

        let now = chrono::Utc::now().naive_utc();
        diesel::update(accounts::table.find(&self.id))
            .set(accounts::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("account soft delete failed: {err}")))?;
        Ok(())
    }

    /// Sum of postings for this account on non-deleted entries.
    pub fn balance(&self, conn: &mut SqliteConnection) -> Result<i64, AppError> {
        let result: Option<i64> = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq(&self.id))
            .filter(entries::deleted_at.is_null())
            .select(diesel::dsl::sum(postings::amount))
            .first(conn)
            .map_err(|err| AppError::Internal(format!("balance query failed: {err}")))?;

        Ok(result.unwrap_or(0))
    }

    /// Balance including all descendant accounts.
    pub fn balance_with_descendants(&self, conn: &mut SqliteConnection) -> Result<i64, AppError> {
        let descendants = self.descendants(conn)?;
        let mut account_ids: Vec<&AccountRef> = descendants.iter().map(|a| &a.id).collect();
        account_ids.push(&self.id);

        let result: Option<i64> = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq_any(&account_ids))
            .filter(entries::deleted_at.is_null())
            .select(diesel::dsl::sum(postings::amount))
            .first(conn)
            .map_err(|err| {
                AppError::Internal(format!("balance_with_descendants query failed: {err}"))
            })?;

        Ok(result.unwrap_or(0))
    }

    /// Balance filtered by entry date <= given date.
    pub fn balance_as_of(&self, date: NaiveDate, conn: &mut SqliteConnection) -> Result<i64, AppError> {
        let result: Option<i64> = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq(&self.id))
            .filter(entries::deleted_at.is_null())
            .filter(entries::date.le(date))
            .select(diesel::dsl::sum(postings::amount))
            .first(conn)
            .map_err(|err| AppError::Internal(format!("balance_as_of query failed: {err}")))?;

        Ok(result.unwrap_or(0))
    }

    /// Balance including descendants, filtered by entry date <= given date.
    pub fn balance_as_of_with_descendants(
        &self,
        date: NaiveDate,
        conn: &mut SqliteConnection,
    ) -> Result<i64, AppError> {
        let descendants = self.descendants(conn)?;
        let mut account_ids: Vec<&AccountRef> = descendants.iter().map(|a| &a.id).collect();
        account_ids.push(&self.id);

        let result: Option<i64> = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq_any(&account_ids))
            .filter(entries::deleted_at.is_null())
            .filter(entries::date.le(date))
            .select(diesel::dsl::sum(postings::amount))
            .first(conn)
            .map_err(|err| {
                AppError::Internal(format!(
                    "balance_as_of_with_descendants query failed: {err}"
                ))
            })?;

        Ok(result.unwrap_or(0))
    }

    /// Balance for a date range (inclusive on both ends).
    pub fn balance_between(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        conn: &mut SqliteConnection,
    ) -> Result<i64, AppError> {
        let result: Option<i64> = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq(&self.id))
            .filter(entries::deleted_at.is_null())
            .filter(entries::date.ge(start_date))
            .filter(entries::date.le(end_date))
            .select(diesel::dsl::sum(postings::amount))
            .first(conn)
            .map_err(|err| AppError::Internal(format!("balance_between query failed: {err}")))?;

        Ok(result.unwrap_or(0))
    }

    /// Balance for a date range, including all descendant accounts.
    pub fn balance_between_with_descendants(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        conn: &mut SqliteConnection,
    ) -> Result<i64, AppError> {
        let descendants = self.descendants(conn)?;
        let mut account_ids: Vec<&AccountRef> = descendants.iter().map(|a| &a.id).collect();
        account_ids.push(&self.id);

        let result: Option<i64> = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq_any(&account_ids))
            .filter(entries::deleted_at.is_null())
            .filter(entries::date.ge(start_date))
            .filter(entries::date.le(end_date))
            .select(diesel::dsl::sum(postings::amount))
            .first(conn)
            .map_err(|err| {
                AppError::Internal(format!(
                    "balance_between_with_descendants query failed: {err}"
                ))
            })?;

        Ok(result.unwrap_or(0))
    }
}

// ── WorkspaceAnchors ─────────────────────────────────────────────────────────
//
// The seven anchor accounts every workspace guarantees. Loaded together so
// downstream report code can take a populated struct rather than re-fetching
// or matching by name. Each field is the active anchor for that role; any
// missing anchor is an Internal error (workspace invariant broken).

#[derive(Debug)]
pub struct WorkspaceAnchors {
    pub assets: Account,
    pub liabilities: Account,
    pub equity: Account,
    pub revenue: Account,
    pub expenses: Account,
    pub cash: Account,
    pub opening_equity: Account,
}

#[derive(Debug, Clone, Copy)]
pub struct BalanceSheetAnchors<'a> {
    pub assets: &'a Account,
    pub liabilities: &'a Account,
    pub equity: &'a Account,
    pub revenue: &'a Account,
    pub expenses: &'a Account,
    pub opening_equity: &'a Account,
}

#[derive(Debug, Clone, Copy)]
pub struct IncomeStatementAnchors<'a> {
    pub revenue: &'a Account,
    pub expenses: &'a Account,
}

#[derive(Debug, Clone, Copy)]
pub struct CashFlowAnchors<'a> {
    pub cash: &'a Account,
    pub opening_equity: &'a Account,
    pub revenue: &'a Account,
    pub expenses: &'a Account,
}

impl WorkspaceAnchors {
    pub fn balance_sheet(&self) -> BalanceSheetAnchors<'_> {
        BalanceSheetAnchors {
            assets: &self.assets,
            liabilities: &self.liabilities,
            equity: &self.equity,
            revenue: &self.revenue,
            expenses: &self.expenses,
            opening_equity: &self.opening_equity,
        }
    }

    pub fn income_statement(&self) -> IncomeStatementAnchors<'_> {
        IncomeStatementAnchors {
            revenue: &self.revenue,
            expenses: &self.expenses,
        }
    }

    pub fn cash_flow(&self) -> CashFlowAnchors<'_> {
        CashFlowAnchors {
            cash: &self.cash,
            opening_equity: &self.opening_equity,
            revenue: &self.revenue,
            expenses: &self.expenses,
        }
    }
}

impl WorkspaceRef {
    pub fn anchors(&self, conn: &mut SqliteConnection) -> Result<WorkspaceAnchors, AppError> {
        let anchored: Vec<Account> = accounts::table
            .filter(accounts::workspace_id.eq(self))
            .filter(accounts::role.is_not_null())
            .filter(accounts::deleted_at.is_null())
            .select(Account::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("anchors query failed: {err}")))?;

        let mut by_role: std::collections::HashMap<AccountRole, Account> = anchored
            .into_iter()
            .filter_map(|account| account.role.map(|role| (role, account)))
            .collect();

        fn take(
            by_role: &mut std::collections::HashMap<AccountRole, Account>,
            role: AccountRole,
        ) -> Result<Account, AppError> {
            by_role.remove(&role).ok_or_else(|| {
                AppError::Internal(format!("workspace missing anchor: {}", role.as_str()))
            })
        }

        Ok(WorkspaceAnchors {
            assets: take(&mut by_role, AccountRole::AssetsRoot)?,
            liabilities: take(&mut by_role, AccountRole::LiabilitiesRoot)?,
            equity: take(&mut by_role, AccountRole::EquityRoot)?,
            revenue: take(&mut by_role, AccountRole::RevenueRoot)?,
            expenses: take(&mut by_role, AccountRole::ExpensesRoot)?,
            cash: take(&mut by_role, AccountRole::Cash)?,
            opening_equity: take(&mut by_role, AccountRole::OpeningEquity)?,
        })
    }
}

impl WorkspaceRef {
    pub fn accounts(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        accounts::table
            .filter(accounts::workspace_id.eq(self))
            .filter(accounts::deleted_at.is_null())
            .select(Account::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("account list failed: {err}")))
    }

    pub fn root_accounts(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        accounts::table
            .filter(accounts::workspace_id.eq(self))
            .filter(accounts::parent_id.is_null())
            .filter(accounts::deleted_at.is_null())
            .select(Account::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("root accounts query failed: {err}")))
    }
}

// ── NewAccount ───────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = accounts)]
struct NewAccountRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    parent_id: Option<AccountRef>,
    name: String,
    account_type: AccountType,
    currency: String,
    role: Option<AccountRole>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewAccount {
    workspace: WorkspaceRef,
    parent_id: Option<AccountRef>,
    name: String,
    account_type: AccountType,
    currency: String,
    role: Option<AccountRole>,
}

/// Anchor roles that may be assigned to non-root accounts.
/// Root anchor roles are derived from AccountType in `anchored_root`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ChildAnchor {
    Cash,
    OpeningEquity,
}

impl ChildAnchor {
    fn role(self) -> AccountRole {
        match self {
            ChildAnchor::Cash => AccountRole::Cash,
            ChildAnchor::OpeningEquity => AccountRole::OpeningEquity,
        }
    }

    fn required_account_type(self) -> AccountType {
        match self {
            ChildAnchor::Cash => AccountType::Asset,
            ChildAnchor::OpeningEquity => AccountType::Equity,
        }
    }
}

impl NewAccount {
    /// Create a child account. Inherits workspace and account_type from parent.
    pub fn child(parent: &Account, name: &str) -> Self {
        NewAccount {
            workspace: parent.workspace.clone(),
            parent_id: Some(parent.id.clone()),
            name: name.to_string(),
            account_type: parent.account_type,
            currency: parent.currency.clone(),
            role: None,
        }
    }

    /// Seed-only: build a root account tagged with the matching root anchor role.
    /// Each (workspace, account_type) gets exactly one of these via `create_default_tree`.
    pub(crate) fn anchored_root(
        workspace: WorkspaceRef,
        name: &str,
        account_type: AccountType,
    ) -> Self {
        let role = match account_type {
            AccountType::Asset => AccountRole::AssetsRoot,
            AccountType::Liability => AccountRole::LiabilitiesRoot,
            AccountType::Equity => AccountRole::EquityRoot,
            AccountType::Revenue => AccountRole::RevenueRoot,
            AccountType::Expense => AccountRole::ExpensesRoot,
        };
        NewAccount {
            workspace,
            parent_id: None,
            name: name.to_string(),
            account_type,
            currency: "CAD".to_string(),
            role: Some(role),
        }
    }

    /// Seed-only: build a child account tagged with a non-root anchor role.
    /// Anchor's required account_type must match the parent's account_type.
    pub(crate) fn anchored_child(parent: &Account, name: &str, anchor: ChildAnchor) -> Self {
        debug_assert_eq!(
            parent.account_type,
            anchor.required_account_type(),
            "ChildAnchor {:?} requires parent of type {:?}, got {:?}",
            anchor,
            anchor.required_account_type(),
            parent.account_type,
        );
        NewAccount {
            workspace: parent.workspace.clone(),
            parent_id: Some(parent.id.clone()),
            name: name.to_string(),
            account_type: parent.account_type,
            currency: parent.currency.clone(),
            role: Some(anchor.role()),
        }
    }

    pub fn currency(mut self, currency: &str) -> Self {
        self.currency = currency.to_string();
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Account, AppError> {
        // If a soft-deleted account with the same (workspace, parent_id, name)
        // exists, restore it instead of inserting a new row. The UNIQUE
        // constraint doesn't exclude soft-deleted rows, so creating a fresh
        // row would conflict; restoring keeps history linked to the same id.
        // Safe to resurrect: soft_delete refuses while active children or
        // entries exist, so the restored row carries no live state.
        let soft_deleted: Option<Account> = {
            let mut query = accounts::table
                .filter(accounts::workspace_id.eq(&self.workspace))
                .filter(accounts::name.eq(&self.name))
                .filter(accounts::deleted_at.is_not_null())
                .into_boxed();
            query = match &self.parent_id {
                Some(parent) => query.filter(accounts::parent_id.eq(parent)),
                None => query.filter(accounts::parent_id.is_null()),
            };
            query
                .select(Account::as_select())
                .first(conn)
                .optional()
                .map_err(|err| {
                    AppError::Internal(format!("soft-deleted lookup failed: {err}"))
                })?
        };

        if let Some(account) = soft_deleted {
            let now = chrono::Utc::now().naive_utc();
            return diesel::update(accounts::table.find(&account.id))
                .set((
                    accounts::deleted_at.eq::<Option<chrono::NaiveDateTime>>(None),
                    accounts::updated_at.eq(now),
                ))
                .returning(Account::as_returning())
                .get_result(conn)
                .map_err(|err| {
                    AppError::Internal(format!("account restore failed: {err}"))
                });
        }

        // For root accounts, enforce uniqueness manually since SQLite
        // treats NULL as distinct in UNIQUE constraints
        if self.parent_id.is_none() {
            let existing: i64 = accounts::table
                .filter(accounts::workspace_id.eq(&self.workspace))
                .filter(accounts::parent_id.is_null())
                .filter(accounts::name.eq(&self.name))
                .filter(accounts::deleted_at.is_null())
                .count()
                .get_result(conn)
                .map_err(|err| {
                    AppError::Internal(format!("root uniqueness check failed: {err}"))
                })?;

            if existing > 0 {
                return Err(AppError::Conflict(format!(
                    "root account '{}' already exists in this workspace",
                    self.name
                )));
            }
        }

        let now = chrono::Utc::now().naive_utc();
        diesel::insert_into(accounts::table)
            .values(NewAccountRow {
                id: Uuid::new_v4().to_string(),
                workspace: self.workspace,
                parent_id: self.parent_id,
                name: self.name.clone(),
                account_type: self.account_type,
                currency: self.currency,
                role: self.role,
                created_at: now,
                updated_at: now,
            })
            .returning(Account::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "account '{}' already exists under this parent",
                        self.name
                    ))
                }
                other => AppError::Internal(format!("account insert failed: {other}")),
            })
    }
}

// ── Default Account Tree ─────────────────────────────────────────────────────

pub fn create_default_tree(
    workspace: &WorkspaceRef,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    let assets =
        NewAccount::anchored_root(workspace.clone(), "Assets", AccountType::Asset).insert(conn)?;
    let liabilities =
        NewAccount::anchored_root(workspace.clone(), "Liabilities", AccountType::Liability)
            .insert(conn)?;
    let equity =
        NewAccount::anchored_root(workspace.clone(), "Equity", AccountType::Equity).insert(conn)?;
    let revenue =
        NewAccount::anchored_root(workspace.clone(), "Revenue", AccountType::Revenue)
            .insert(conn)?;
    let expenses =
        NewAccount::anchored_root(workspace.clone(), "Expenses", AccountType::Expense)
            .insert(conn)?;

    // Asset sub-accounts (Chequing is the cash anchor)
    NewAccount::anchored_child(&assets, "Chequing", ChildAnchor::Cash).insert(conn)?;
    NewAccount::child(&assets, "Savings").insert(conn)?;
    NewAccount::child(&assets, "TFSA").insert(conn)?;
    NewAccount::child(&assets, "RRSP").insert(conn)?;

    // Liability sub-accounts
    NewAccount::child(&liabilities, "Credit Card").insert(conn)?;

    // Revenue sub-accounts
    NewAccount::child(&revenue, "Employment").insert(conn)?;

    // Expense sub-accounts
    NewAccount::child(&expenses, "Food").insert(conn)?;
    NewAccount::child(&expenses, "Rent").insert(conn)?;
    NewAccount::child(&expenses, "Groceries").insert(conn)?;
    NewAccount::child(&expenses, "Dining").insert(conn)?;
    NewAccount::child(&expenses, "Transit").insert(conn)?;
    NewAccount::child(&expenses, "Utilities").insert(conn)?;
    NewAccount::child(&expenses, "Entertainment").insert(conn)?;

    // Equity sub-accounts (Opening Balances is the opening-equity anchor)
    NewAccount::anchored_child(&equity, "Opening Balances", ChildAnchor::OpeningEquity)
        .insert(conn)?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entry::{NewEntry, PostingInput};
    use crate::domain::user::NewUser;
    use crate::domain::workspace::{NewWorkspace, Workspace};
    use crate::test_helpers::setup_test_db;

    /// Setup creates a workspace with default account tree already populated.
    fn setup() -> (SqliteConnection, Workspace) {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace = NewWorkspace::new("My Finances", user.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        (conn, workspace)
    }

    /// Helper to find an account by name in workspace.
    fn find_account(name: &str, workspace: &Workspace, conn: &mut SqliteConnection) -> Account {
        let all = workspace.as_ref().accounts(conn).unwrap();
        all.into_iter().find(|a| a.name == name).unwrap()
    }

    #[test]
    fn default_tree_created_on_workspace_creation() {
        let (mut conn, workspace) = setup();

        let roots = workspace.as_ref().root_accounts(&mut conn).unwrap();
        let root_names: Vec<&str> = roots.iter().map(|a| a.name.as_str()).collect();
        assert!(root_names.contains(&"Assets"));
        assert!(root_names.contains(&"Liabilities"));
        assert!(root_names.contains(&"Equity"));
        assert!(root_names.contains(&"Revenue"));
        assert!(root_names.contains(&"Expenses"));
        assert_eq!(roots.len(), 5);

        let all = workspace.as_ref().accounts(&mut conn).unwrap();
        // 5 roots + 4 asset + 1 liability + 1 revenue + 7 expense + 1 equity = 19
        assert_eq!(all.len(), 19);

        let chequing = all.iter().find(|a| a.name == "Chequing").unwrap();
        assert_eq!(chequing.account_type, AccountType::Asset);
        assert_eq!(chequing.role, Some(AccountRole::Cash));

        let opening = all.iter().find(|a| a.name == "Opening Balances").unwrap();
        assert_eq!(opening.role, Some(AccountRole::OpeningEquity));

        let assets_root = roots.iter().find(|a| a.name == "Assets").unwrap();
        assert_eq!(assets_root.role, Some(AccountRole::AssetsRoot));

        // Non-anchor accounts have no role.
        let savings = all.iter().find(|a| a.name == "Savings").unwrap();
        assert_eq!(savings.role, None);
    }

    #[test]
    fn account_role_string_round_trip() {
        for role in [
            AccountRole::AssetsRoot,
            AccountRole::LiabilitiesRoot,
            AccountRole::EquityRoot,
            AccountRole::RevenueRoot,
            AccountRole::ExpensesRoot,
            AccountRole::Cash,
            AccountRole::OpeningEquity,
        ] {
            assert_eq!(AccountRole::from_str(role.as_str()), Some(role));
        }
        assert_eq!(AccountRole::from_str("bogus"), None);
    }

    #[test]
    fn newly_created_account_has_no_role() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        let brokerage = NewAccount::child(&assets, "Brokerage")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(brokerage.role, None);
    }

    #[test]
    fn create_child_account_succeeds() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        let child = NewAccount::child(&assets, "Brokerage")
            .insert(&mut conn)
            .expect("child creation should succeed");

        assert_eq!(child.name, "Brokerage");
        assert_eq!(child.account_type, AccountType::Asset);
        assert_eq!(child.parent_id, Some(assets.id.clone()));
        assert!(!child.is_root());
    }

    #[test]
    fn create_deeply_nested_account() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let dining_out = NewAccount::child(&food, "Dining Out")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(dining_out.account_type, AccountType::Expense);
        assert_eq!(dining_out.parent_id, Some(food.id.clone()));

        // Verify full_name goes 3 levels deep
        assert_eq!(
            dining_out.full_name(&mut conn).unwrap(),
            "Expenses:Food:Dining Out"
        );
    }

    #[test]
    fn sibling_name_uniqueness_enforced() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        // "Chequing" already exists as child of Assets from defaults
        let err = NewAccount::child(&assets, "Chequing")
            .insert(&mut conn)
            .expect_err("duplicate sibling name should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn cousin_name_duplication_allowed() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        let bank_a = NewAccount::child(&assets, "Bank A")
            .insert(&mut conn)
            .unwrap();
        let bank_b = NewAccount::child(&assets, "Bank B")
            .insert(&mut conn)
            .unwrap();

        NewAccount::child(&bank_a, "Main")
            .insert(&mut conn)
            .unwrap();
        NewAccount::child(&bank_b, "Main")
            .insert(&mut conn)
            .expect("cousin duplication should succeed");
    }

    #[test]
    fn anchors_loaded_from_seeded_workspace() {
        let (mut conn, workspace) = setup();
        let anchors = workspace.as_ref().anchors(&mut conn).unwrap();

        assert_eq!(anchors.assets.name, "Assets");
        assert_eq!(anchors.assets.role, Some(AccountRole::AssetsRoot));
        assert_eq!(anchors.liabilities.name, "Liabilities");
        assert_eq!(anchors.equity.name, "Equity");
        assert_eq!(anchors.revenue.name, "Revenue");
        assert_eq!(anchors.expenses.name, "Expenses");
        assert_eq!(anchors.cash.name, "Chequing");
        assert_eq!(anchors.cash.role, Some(AccountRole::Cash));
        assert_eq!(anchors.opening_equity.name, "Opening Balances");
        assert_eq!(anchors.opening_equity.role, Some(AccountRole::OpeningEquity));
    }

    #[test]
    fn anchors_unaffected_by_rename() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        chequing.rename("Wealthsimple Cash", &mut conn).unwrap();

        let anchors = workspace.as_ref().anchors(&mut conn).unwrap();
        assert_eq!(anchors.cash.name, "Wealthsimple Cash");
        assert_eq!(anchors.cash.role, Some(AccountRole::Cash));
    }

    #[test]
    fn anchors_fail_when_role_missing() {
        let (mut conn, workspace) = setup();
        // Clear the Cash anchor's role directly to simulate a broken workspace.
        let chequing = find_account("Chequing", &workspace, &mut conn);
        diesel::update(accounts::table.find(&chequing.id))
            .set(accounts::role.eq::<Option<AccountRole>>(None))
            .execute(&mut conn)
            .unwrap();

        let err = workspace
            .as_ref()
            .anchors(&mut conn)
            .expect_err("missing anchor should fail");
        assert!(matches!(err, AppError::Internal(ref msg) if msg.contains("cash")));
    }

    #[test]
    fn duplicate_root_anchor_rejected() {
        let (mut conn, workspace) = setup();
        // Seed already created an Asset root with AssetsRoot anchor role.
        let err = NewAccount::anchored_root(
            workspace.as_ref().clone(),
            "Extra Assets",
            AccountType::Asset,
        )
        .insert(&mut conn)
        .expect_err("second AssetsRoot in same workspace should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_root_name_different_workspaces() {
        let mut conn = setup_test_db();
        let user_a = NewUser::new("a@example.com", "A", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let user_b = NewUser::new("b@example.com", "B", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace_a = NewWorkspace::new("WS A", user_a.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        let workspace_b = NewWorkspace::new("WS B", user_b.as_ref().clone())
            .insert(&mut conn)
            .unwrap();

        // Both workspaces already have "Assets" from defaults — this test passes
        // if create_default_tree didn't error (which it didn't, since setup succeeded)
        let roots_a = workspace_a.as_ref().root_accounts(&mut conn).unwrap();
        let roots_b = workspace_b.as_ref().root_accounts(&mut conn).unwrap();
        assert!(roots_a.iter().any(|a| a.name == "Assets"));
        assert!(roots_b.iter().any(|a| a.name == "Assets"));
    }

    #[test]
    fn full_name_returns_colon_delimited_path() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let cafes = NewAccount::child(&food, "Cafés")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(expenses.full_name(&mut conn).unwrap(), "Expenses");
        assert_eq!(food.full_name(&mut conn).unwrap(), "Expenses:Food");
        assert_eq!(
            cafes.full_name(&mut conn).unwrap(),
            "Expenses:Food:Cafés"
        );
    }

    #[test]
    fn ancestors_returns_root_to_parent_chain() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);
        let bank = NewAccount::child(&assets, "Bank")
            .insert(&mut conn)
            .unwrap();
        let chequing = NewAccount::child(&bank, "Main Chequing")
            .insert(&mut conn)
            .unwrap();

        let ancestors = chequing.ancestors(&mut conn).unwrap();
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].name, "Assets");
        assert_eq!(ancestors[1].name, "Bank");

        let root_ancestors = assets.ancestors(&mut conn).unwrap();
        assert_eq!(root_ancestors.len(), 0);
    }

    #[test]
    fn children_returns_direct_children() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        // Assets has default children: Chequing, Savings, TFSA, RRSP
        let children = assets.children(&mut conn).unwrap();
        assert_eq!(children.len(), 4);
    }

    #[test]
    fn descendants_returns_all_transitive_children() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        // Add sub-categories under Food
        NewAccount::child(&food, "Cafés").insert(&mut conn).unwrap();
        NewAccount::child(&food, "Fast Food")
            .insert(&mut conn)
            .unwrap();

        let descendants = expenses.descendants(&mut conn).unwrap();
        // Default expense children: Food, Rent, Groceries, Dining, Transit, Utilities, Entertainment (7)
        // Plus Food's new children: Cafés, Fast Food (2)
        assert_eq!(descendants.len(), 9);
    }

    #[test]
    fn account_type_inherited_from_root() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);
        let bank = NewAccount::child(&assets, "Bank")
            .insert(&mut conn)
            .unwrap();
        let grandchild = NewAccount::child(&bank, "Premium")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(assets.account_type, AccountType::Asset);
        assert_eq!(bank.account_type, AccountType::Asset);
        assert_eq!(grandchild.account_type, AccountType::Asset);
    }

    #[test]
    fn soft_delete_leaf_succeeds() {
        let (mut conn, workspace) = setup();
        let savings = find_account("Savings", &workspace, &mut conn);

        savings.soft_delete(&mut conn).unwrap();

        let assets = find_account("Assets", &workspace, &mut conn);
        let children = assets.children(&mut conn).unwrap();
        assert!(!children.iter().any(|a| a.name == "Savings"));
    }

    #[test]
    fn soft_delete_anchor_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        let err = chequing
            .soft_delete(&mut conn)
            .expect_err("deleting anchor account should fail");
        assert!(matches!(err, AppError::Conflict(_)));

        let opening = find_account("Opening Balances", &workspace, &mut conn);
        let err = opening
            .soft_delete(&mut conn)
            .expect_err("deleting anchor account should fail");
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn soft_delete_with_active_children_rejected() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        // Assets has children, so delete should fail
        let err = assets
            .soft_delete(&mut conn)
            .expect_err("delete with active children should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn soft_delete_with_all_deleted_children_succeeds() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        // Build a non-anchor branch we can fully soft-delete.
        let custom = NewAccount::child(&assets, "Custom").insert(&mut conn).unwrap();
        let sub = NewAccount::child(&custom, "Sub").insert(&mut conn).unwrap();

        sub.soft_delete(&mut conn).unwrap();
        custom
            .soft_delete(&mut conn)
            .expect("delete with all children deleted should succeed");
    }

    #[test]
    fn accounts_returns_only_active() {
        let (mut conn, workspace) = setup();
        let tfsa = find_account("TFSA", &workspace, &mut conn);

        tfsa.soft_delete(&mut conn).unwrap();

        let all = workspace.as_ref().accounts(&mut conn).unwrap();
        assert!(!all.iter().any(|a| a.name == "TFSA"));
        assert!(all.iter().any(|a| a.name == "Savings"));
    }

    #[test]
    fn root_accounts_returns_only_roots() {
        let (mut conn, workspace) = setup();

        let roots = workspace.as_ref().root_accounts(&mut conn).unwrap();
        assert!(roots.iter().all(|a| a.is_root()));
        assert!(roots.iter().any(|a| a.name == "Assets"));
        assert!(!roots.iter().any(|a| a.name == "Chequing"));
    }

    #[test]
    fn custom_currency_on_child() {
        let (mut conn, workspace) = setup();
        let assets = find_account("Assets", &workspace, &mut conn);

        let account = NewAccount::child(&assets, "USD Savings")
            .currency("USD")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(account.currency, "USD");
    }

    // ── Balance tests ───────────────────────────────────────────────────

    fn nd(s: &str) -> chrono::NaiveDate {
        s.parse().unwrap()
    }

    fn make_entry(
        workspace: &Workspace,
        from: &Account,
        to: &Account,
        amount: i32,
        date: &str,
        conn: &mut SqliteConnection,
    ) -> crate::domain::entry::Entry {
        NewEntry::new(
            workspace.as_ref().clone(),
            nd(date),
            "test",
            vec![
                PostingInput::new(from.id.clone(), -amount),
                PostingInput::new(to.id.clone(), amount),
            ],
        )
        .insert(conn)
        .unwrap()
    }

    #[test]
    fn balance_sum_of_postings() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &food, 800, "2026-01-16", &mut conn);

        assert_eq!(food.balance(&mut conn).unwrap(), 2000);
        assert_eq!(chequing.balance(&mut conn).unwrap(), -2000);
    }

    #[test]
    fn balance_excludes_soft_deleted_entries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        let deleted_entry = make_entry(&workspace, &chequing, &food, 800, "2026-01-16", &mut conn);
        deleted_entry.soft_delete(&mut conn).unwrap();

        assert_eq!(food.balance(&mut conn).unwrap(), 1200);
    }

    #[test]
    fn balance_with_descendants_includes_children() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let expenses = find_account("Expenses", &workspace, &mut conn);

        let cafes = NewAccount::child(&food, "Cafes").insert(&mut conn).unwrap();

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-01-16", &mut conn);

        // Food direct balance = 1200
        assert_eq!(food.balance(&mut conn).unwrap(), 1200);

        // Expenses with descendants = Food(1200) + Cafes(500) = 1700
        // (other expense children have no postings)
        assert_eq!(expenses.balance_with_descendants(&mut conn).unwrap(), 1700);
    }

    #[test]
    fn balance_as_of_filters_by_date() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &food, 800, "2026-01-20", &mut conn);

        assert_eq!(food.balance_as_of(nd("2026-01-15"), &mut conn).unwrap(), 1200);
        assert_eq!(food.balance_as_of(nd("2026-01-20"), &mut conn).unwrap(), 2000);
        assert_eq!(food.balance_as_of(nd("2026-01-14"), &mut conn).unwrap(), 0);
    }

    #[test]
    fn fundamental_equation_holds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        // Earn salary: debit chequing, credit employment
        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);

        // Spend on food
        make_entry(&workspace, &chequing, &food, 5000, "2026-01-05", &mut conn);

        // Pay rent
        make_entry(&workspace, &chequing, &rent, 150000, "2026-01-05", &mut conn);

        let assets = find_account("Assets", &workspace, &mut conn);
        let liabilities = find_account("Liabilities", &workspace, &mut conn);
        let equity = find_account("Equity", &workspace, &mut conn);
        let revenue = find_account("Revenue", &workspace, &mut conn);
        let expenses = find_account("Expenses", &workspace, &mut conn);

        let asset_balance = assets.balance_with_descendants(&mut conn).unwrap();
        let liability_balance = liabilities.balance_with_descendants(&mut conn).unwrap();
        let equity_balance = equity.balance_with_descendants(&mut conn).unwrap();
        let revenue_balance = revenue.balance_with_descendants(&mut conn).unwrap();
        let expense_balance = expenses.balance_with_descendants(&mut conn).unwrap();

        // Sum of all postings across all accounts = 0 (guaranteed by zero-sum invariant)
        // A + L + E + R + X = 0
        let total = asset_balance + liability_balance + equity_balance + revenue_balance + expense_balance;
        assert_eq!(total, 0, "fundamental equation violated: A={asset_balance} L={liability_balance} E={equity_balance} R={revenue_balance} X={expense_balance}");
    }

    #[test]
    fn account_with_active_postings_cannot_be_deleted() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let err = food.soft_delete(&mut conn).expect_err("should fail with active postings");
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn delete_then_readd_account_restores_same_id() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);
        let original_id = food.id.clone();
        let expenses = find_account("Expenses", &workspace, &mut conn);

        food.soft_delete(&mut conn).unwrap();

        let recreated = NewAccount::child(&expenses, "Food").insert(&mut conn).unwrap();

        assert_eq!(recreated.id, original_id);
        assert!(recreated.deleted_at.is_none());
    }

    #[test]
    fn create_delete_create_user_account_restores_same_id() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);

        let first = NewAccount::child(&expenses, "test").insert(&mut conn).unwrap();
        let original_id = first.id.clone();

        first.soft_delete(&mut conn).unwrap();

        let recreated = NewAccount::child(&expenses, "test")
            .insert(&mut conn)
            .expect("recreation after soft delete must succeed without conflict");

        assert_eq!(recreated.id, original_id);
        assert!(recreated.deleted_at.is_none());
    }

    #[test]
    fn account_with_only_deleted_entry_postings_can_be_deleted() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        entry.soft_delete(&mut conn).unwrap();

        food.soft_delete(&mut conn).expect("should succeed when all entries deleted");
    }

    #[test]
    fn balance_returns_zero_for_no_postings() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        assert_eq!(chequing.balance(&mut conn).unwrap(), 0);
    }

    // ── Rename tests ───────────────────────────────────────────────

    #[test]
    fn rename_succeeds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        let renamed = chequing.rename("Main Chequing", &mut conn).unwrap();

        assert_eq!(renamed.name, "Main Chequing");
        assert_eq!(renamed.id, chequing.id);
    }

    #[test]
    fn rename_duplicate_under_same_parent_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        // "Savings" is a sibling under Assets
        let result = chequing.rename("Savings", &mut conn);

        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    // ── Reparent tests ─────────────────────────────────────────────

    #[test]
    fn reparent_succeeds() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        // Create a sub-category under Expenses to move Food into
        let categories = NewAccount::child(&expenses, "Categories")
            .insert(&mut conn)
            .unwrap();

        let reparented = food.reparent(&categories.id, &mut conn).unwrap();

        assert_eq!(reparented.parent_id, Some(categories.id.clone()));
        assert_eq!(
            reparented.full_name(&mut conn).unwrap(),
            "Expenses:Categories:Food"
        );
    }

    #[test]
    fn reparent_cycle_rejected() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        // Try to reparent Expenses under Food (its own descendant)
        let result = expenses.reparent(&food.id, &mut conn);

        // Expenses is a root, so it should be rejected for that reason first
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_cycle_rejected_non_root() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        // Create a child of Food
        let fast_food = NewAccount::child(&food, "Fast Food")
            .insert(&mut conn)
            .unwrap();

        // Try to reparent Food under its own child
        let result = food.reparent(&fast_food.id, &mut conn);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_cross_type_rejected() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);
        let assets = find_account("Assets", &workspace, &mut conn);

        // Try to move an Expense account under Assets
        let result = food.reparent(&assets.id, &mut conn);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_root_account_rejected() {
        let (mut conn, workspace) = setup();
        let expenses = find_account("Expenses", &workspace, &mut conn);
        let revenue = find_account("Revenue", &workspace, &mut conn);

        // Try to reparent "Expenses" root
        let result = expenses.reparent(&revenue.id, &mut conn);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_to_self_rejected() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        let result = food.reparent(&food.id, &mut conn);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_cross_workspace_rejected() {
        let mut conn = setup_test_db();
        let user_a = NewUser::new("a@example.com", "A", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let user_b = NewUser::new("b@example.com", "B", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace_a = NewWorkspace::new("WS A", user_a.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        let workspace_b = NewWorkspace::new("WS B", user_b.as_ref().clone())
            .insert(&mut conn)
            .unwrap();

        let food_a = find_account("Food", &workspace_a, &mut conn);
        let expenses_b = find_account("Expenses", &workspace_b, &mut conn);

        let result = food_a.reparent(&expenses_b.id, &mut conn);

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    // ── balance_between tests ──────────────────────────────────────

    #[test]
    fn balance_between_returns_sum_in_date_range() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1000, "2026-01-10", &mut conn);
        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &food, 800, "2026-01-20", &mut conn);

        // Range captures only middle entry
        assert_eq!(food.balance_between(nd("2026-01-11"), nd("2026-01-19"), &mut conn).unwrap(), 1200);
    }

    #[test]
    fn balance_between_inclusive_boundaries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        assert_eq!(food.balance_between(nd("2026-01-15"), nd("2026-01-15"), &mut conn).unwrap(), 1200);
    }

    #[test]
    fn balance_between_returns_zero_for_empty_range() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        assert_eq!(food.balance_between(nd("2026-02-01"), nd("2026-02-28"), &mut conn).unwrap(), 0);
    }

    #[test]
    fn balance_between_with_descendants_includes_children() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let expenses = find_account("Expenses", &workspace, &mut conn);

        let cafes = NewAccount::child(&food, "Cafes").insert(&mut conn).unwrap();

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-01-16", &mut conn);

        assert_eq!(
            expenses.balance_between_with_descendants(nd("2026-01-01"), nd("2026-01-31"), &mut conn).unwrap(),
            1700
        );
    }

    #[test]
    fn balance_as_of_with_descendants_filters_by_date() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let assets = find_account("Assets", &workspace, &mut conn);

        let cafes = NewAccount::child(&food, "Cafes").insert(&mut conn).unwrap();

        // Entries at different dates on different accounts
        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-02-10", &mut conn);

        // As of Jan 31: chequing has -1200 (only first entry), assets tree = -1200
        assert_eq!(assets.balance_as_of_with_descendants(nd("2026-01-31"), &mut conn).unwrap(), -1200);
        // As of Feb 28: chequing has -1700, assets tree = -1700
        assert_eq!(assets.balance_as_of_with_descendants(nd("2026-02-28"), &mut conn).unwrap(), -1700);
    }
}
