use chrono::NaiveDateTime;
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

// ── Currency ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Currency {
    CAD,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Currency::CAD => "CAD",
        })
    }
}

impl std::str::FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAD" => Ok(Currency::CAD),
            other => Err(format!("unknown currency: {other}")),
        }
    }
}

// ── Category ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

// ── SystemAccount ────────────────────────────────────────────────────────────

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = Text)]
pub enum SystemAccount {
    AssetsRoot,
    LiabilitiesRoot,
    EquityRoot,
    RevenueRoot,
    ExpensesRoot,
    Chequing,
    Savings,
    RetainedEarnings,
    OpeningBalances,
}

impl SystemAccount {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemAccount::AssetsRoot => "AssetsRoot",
            SystemAccount::LiabilitiesRoot => "LiabilitiesRoot",
            SystemAccount::EquityRoot => "EquityRoot",
            SystemAccount::RevenueRoot => "RevenueRoot",
            SystemAccount::ExpensesRoot => "ExpensesRoot",
            SystemAccount::Chequing => "Chequing",
            SystemAccount::Savings => "Savings",
            SystemAccount::RetainedEarnings => "RetainedEarnings",
            SystemAccount::OpeningBalances => "OpeningBalances",
        }
    }

    pub fn category(&self) -> Category {
        match self {
            SystemAccount::AssetsRoot
            | SystemAccount::Chequing
            | SystemAccount::Savings => Category::Asset,
            SystemAccount::LiabilitiesRoot => Category::Liability,
            SystemAccount::EquityRoot
            | SystemAccount::RetainedEarnings
            | SystemAccount::OpeningBalances => Category::Equity,
            SystemAccount::RevenueRoot => Category::Revenue,
            SystemAccount::ExpensesRoot => Category::Expense,
        }
    }

    pub fn parent_variant(&self) -> Option<SystemAccount> {
        match self {
            SystemAccount::AssetsRoot
            | SystemAccount::LiabilitiesRoot
            | SystemAccount::EquityRoot
            | SystemAccount::RevenueRoot
            | SystemAccount::ExpensesRoot => None,
            SystemAccount::Chequing | SystemAccount::Savings => {
                Some(SystemAccount::AssetsRoot)
            }
            SystemAccount::RetainedEarnings | SystemAccount::OpeningBalances => {
                Some(SystemAccount::EquityRoot)
            }
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent_variant().is_none()
    }
}

impl std::fmt::Display for SystemAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SystemAccount::AssetsRoot => "Assets",
            SystemAccount::LiabilitiesRoot => "Liabilities",
            SystemAccount::EquityRoot => "Equity",
            SystemAccount::RevenueRoot => "Revenue",
            SystemAccount::ExpensesRoot => "Expenses",
            SystemAccount::Chequing => "Chequing",
            SystemAccount::Savings => "Savings",
            SystemAccount::RetainedEarnings => "Retained Earnings",
            SystemAccount::OpeningBalances => "Opening Balances",
        })
    }
}

impl std::str::FromStr for SystemAccount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AssetsRoot" => Ok(SystemAccount::AssetsRoot),
            "LiabilitiesRoot" => Ok(SystemAccount::LiabilitiesRoot),
            "EquityRoot" => Ok(SystemAccount::EquityRoot),
            "RevenueRoot" => Ok(SystemAccount::RevenueRoot),
            "ExpensesRoot" => Ok(SystemAccount::ExpensesRoot),
            "Chequing" => Ok(SystemAccount::Chequing),
            "Savings" => Ok(SystemAccount::Savings),
            "RetainedEarnings" => Ok(SystemAccount::RetainedEarnings),
            "OpeningBalances" => Ok(SystemAccount::OpeningBalances),
            other => Err(format!("unknown system account: {other}")),
        }
    }
}

impl FromSql<Text, Sqlite> for SystemAccount {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        s.parse().map_err(|e: String| e.into())
    }
}

impl ToSql<Text, Sqlite> for SystemAccount {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(self.as_str(), out)
    }
}

// ── CustomAccount ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomAccount {
    pub parent: Box<AccountType>,
    pub name: String,
}

impl CustomAccount {
    pub fn category(&self) -> Category {
        self.parent.category()
    }
}

// ── AccountType ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AccountType {
    System(SystemAccount),
    Custom(CustomAccount),
}

impl AccountType {
    pub fn category(&self) -> Category {
        match self {
            AccountType::System(s) => s.category(),
            AccountType::Custom(c) => c.category(),
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::System(s) => write!(f, "{s}"),
            AccountType::Custom(c) => f.write_str(&c.name),
        }
    }
}

// ── AccountRef ───────────────────────────────────────────────────────────────

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

// ── Diesel row (internal) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct AccountRow {
    id: AccountRef,
    workspace_id: WorkspaceRef,
    parent_id: Option<AccountRef>,
    name: Option<String>,
    account_type: String,
    currency: String,
    deleted_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = accounts)]
struct NewAccountRow {
    id: String,
    workspace_id: WorkspaceRef,
    parent_id: Option<AccountRef>,
    name: Option<String>,
    account_type: String,
    currency: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

// ── Account (domain) ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Account {
    pub id: AccountRef,
    pub workspace: WorkspaceRef,
    pub parent_id: Option<AccountRef>,
    pub account_type: AccountType,
    pub currency: Currency,
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
        let row: Option<AccountRow> = accounts::table
            .find(account_ref)
            .select(AccountRow::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("account lookup failed: {err}")))?;

        match row {
            Some(r) => Ok(Some(hydrate(r, conn)?)),
            None => Ok(None),
        }
    }

    pub fn from_system(
        workspace: &WorkspaceRef,
        system: SystemAccount,
        conn: &mut SqliteConnection,
    ) -> Result<Account, AppError> {
        workspace
            .system_account(system, conn)?
            .ok_or_else(|| AppError::NotFound(format!("system account {system:?}")))
    }

    pub fn from_custom_name(
        workspace: &WorkspaceRef,
        name: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Account, AppError> {
        workspace
            .accounts(conn)?
            .into_iter()
            .find(|a| a.name() == name)
            .ok_or_else(|| AppError::NotFound(format!("custom account '{name}'")))
    }

    pub fn name(&self) -> String {
        self.account_type.to_string()
    }

    pub fn category(&self) -> Category {
        self.account_type.category()
    }

    /// Returns the colon-delimited full path from root to this account.
    pub fn full_name(&self, conn: &mut SqliteConnection) -> Result<String, AppError> {
        let ancestors = self.ancestors(conn)?;
        let mut parts: Vec<String> = ancestors.iter().map(|a| a.name()).collect();
        parts.push(self.name());
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
        let rows: Vec<AccountRow> = accounts::table
            .filter(accounts::parent_id.eq(&self.id))
            .filter(accounts::deleted_at.is_null())
            .select(AccountRow::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("children query failed: {err}")))?;

        rows.into_iter().map(|r| hydrate(r, conn)).collect()
    }

    /// Returns all transitive descendants (BFS).
    pub fn descendants(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        let mut result = Vec::new();
        let mut queue = vec![self.id.clone()];

        while let Some(parent_id) = queue.pop() {
            let rows: Vec<AccountRow> = accounts::table
                .filter(accounts::parent_id.eq(&parent_id))
                .filter(accounts::deleted_at.is_null())
                .select(AccountRow::as_select())
                .load(conn)
                .map_err(|err| AppError::Internal(format!("descendants query failed: {err}")))?;

            for row in rows {
                let child = hydrate(row, conn)?;
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
        if !matches!(self.account_type, AccountType::Custom(_)) {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "name".into(),
                message: "system accounts cannot be renamed".into(),
            }]));
        }

        let now = chrono::Utc::now().naive_utc();
        diesel::update(accounts::table.find(&self.id))
            .set((
                accounts::name.eq(Some(new_name)),
                accounts::updated_at.eq(now),
            ))
            .execute(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "account '{new_name}' already exists under this parent"
                    ))
                }
                other => AppError::Internal(format!("account rename failed: {other}")),
            })?;

        Account::from_ref(&self.id, conn)?
            .ok_or_else(|| AppError::Internal("account disappeared after rename".to_string()))
    }

    pub fn reparent(
        &self,
        new_parent: &AccountRef,
        conn: &mut SqliteConnection,
    ) -> Result<Account, AppError> {
        if !matches!(self.account_type, AccountType::Custom(_)) {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: "system accounts cannot be reparented".into(),
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

        if parent.category() != self.category() {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "parent_id".into(),
                message: format!(
                    "cannot reparent {:?} account under {:?} account",
                    self.category(),
                    parent.category()
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

        let active_posting_count: i64 = postings::table
            .inner_join(entries::table.on(entries::id.eq(postings::entry_id)))
            .filter(postings::account_id.eq(&self.id))
            .filter(entries::deleted_at.is_null())
            .count()
            .get_result(conn)
            .map_err(|err| AppError::Internal(format!("posting count failed: {err}")))?;

        if active_posting_count > 0 {
            return Err(AppError::Conflict(
                "cannot delete account with active postings".to_string(),
            ));
        }

        let now = chrono::Utc::now().naive_utc();
        diesel::update(accounts::table.find(&self.id))
            .set(accounts::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("account soft delete failed: {err}")))?;
        Ok(())
    }

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

    pub fn balance_as_of(&self, date: &str, conn: &mut SqliteConnection) -> Result<i64, AppError> {
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

    pub fn balance_as_of_with_descendants(
        &self,
        date: &str,
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

    pub fn balance_between(
        &self,
        start_date: &str,
        end_date: &str,
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

    pub fn balance_between_with_descendants(
        &self,
        start_date: &str,
        end_date: &str,
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

impl WorkspaceRef {
    pub fn accounts(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        let rows: Vec<AccountRow> = accounts::table
            .filter(accounts::workspace_id.eq(self))
            .filter(accounts::deleted_at.is_null())
            .select(AccountRow::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("account list failed: {err}")))?;

        rows.into_iter().map(|r| hydrate(r, conn)).collect()
    }

    pub fn root_accounts(&self, conn: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        let rows: Vec<AccountRow> = accounts::table
            .filter(accounts::workspace_id.eq(self))
            .filter(accounts::parent_id.is_null())
            .filter(accounts::deleted_at.is_null())
            .select(AccountRow::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("root accounts query failed: {err}")))?;

        rows.into_iter().map(|r| hydrate(r, conn)).collect()
    }

    pub fn system_account(
        &self,
        system: SystemAccount,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Account>, AppError> {
        let row: Option<AccountRow> = accounts::table
            .filter(accounts::workspace_id.eq(self))
            .filter(accounts::account_type.eq(system.as_str()))
            .filter(accounts::deleted_at.is_null())
            .select(AccountRow::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("system account lookup failed: {err}")))?;

        match row {
            Some(r) => Ok(Some(hydrate(r, conn)?)),
            None => Ok(None),
        }
    }
}

// ── Hydration ────────────────────────────────────────────────────────────────

fn hydrate(row: AccountRow, conn: &mut SqliteConnection) -> Result<Account, AppError> {
    let account_type = parse_account_type(&row, conn)?;
    let currency = row
        .currency
        .parse::<Currency>()
        .map_err(AppError::Internal)?;

    Ok(Account {
        id: row.id,
        workspace: row.workspace_id,
        parent_id: row.parent_id,
        account_type,
        currency,
        deleted_at: row.deleted_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn parse_account_type(
    row: &AccountRow,
    conn: &mut SqliteConnection,
) -> Result<AccountType, AppError> {
    if row.account_type == "Custom" {
        let parent_ref = row.parent_id.as_ref().ok_or_else(|| {
            AppError::Internal(format!("custom account {} missing parent_id", row.id))
        })?;
        let name = row.name.clone().ok_or_else(|| {
            AppError::Internal(format!("custom account {} missing name", row.id))
        })?;
        let parent_row: AccountRow = accounts::table
            .find(parent_ref)
            .select(AccountRow::as_select())
            .first(conn)
            .map_err(|err| AppError::Internal(format!("parent lookup failed: {err}")))?;
        let parent_type = parse_account_type(&parent_row, conn)?;
        Ok(AccountType::Custom(CustomAccount {
            parent: Box::new(parent_type),
            name,
        }))
    } else {
        let system: SystemAccount = row
            .account_type
            .parse()
            .map_err(|e: String| AppError::Internal(e))?;
        Ok(AccountType::System(system))
    }
}

// ── NewCustomAccount ─────────────────────────────────────────────────────────

pub struct NewCustomAccount {
    workspace: WorkspaceRef,
    parent_id: AccountRef,
    name: String,
    currency: Currency,
}

impl NewCustomAccount {
    pub fn child(parent: &Account, name: &str) -> Self {
        Self {
            workspace: parent.workspace.clone(),
            parent_id: parent.id.clone(),
            name: name.to_string(),
            currency: parent.currency,
        }
    }

    pub fn currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Account, AppError> {
        let now = chrono::Utc::now().naive_utc();
        let id = Uuid::new_v4().to_string();
        let account_ref = AccountRef(id.clone());

        diesel::insert_into(accounts::table)
            .values(NewAccountRow {
                id,
                workspace_id: self.workspace,
                parent_id: Some(self.parent_id),
                name: Some(self.name.clone()),
                account_type: "Custom".to_string(),
                currency: self.currency.to_string(),
                created_at: now,
                updated_at: now,
            })
            .execute(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "account '{}' already exists under this parent",
                        self.name
                    ))
                }
                other => AppError::Internal(format!("account insert failed: {other}")),
            })?;

        Account::from_ref(&account_ref, conn)?
            .ok_or_else(|| AppError::Internal("account disappeared after insert".into()))
    }
}

// ── NewSystemAccount ─────────────────────────────────────────────────────────

pub struct NewSystemAccount {
    workspace: WorkspaceRef,
    system: SystemAccount,
    parent_id: Option<AccountRef>,
}

impl NewSystemAccount {
    pub fn new(workspace: &WorkspaceRef, system: SystemAccount) -> Self {
        Self {
            workspace: workspace.clone(),
            system,
            parent_id: None,
        }
    }

    pub fn parent(mut self, parent: &Account) -> Self {
        self.parent_id = Some(parent.id.clone());
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Account, AppError> {
        let now = chrono::Utc::now().naive_utc();
        let id = Uuid::new_v4().to_string();
        let account_ref = AccountRef(id.clone());

        diesel::insert_into(accounts::table)
            .values(NewAccountRow {
                id,
                workspace_id: self.workspace,
                parent_id: self.parent_id,
                name: None,
                account_type: self.system.as_str().to_string(),
                currency: Currency::CAD.to_string(),
                created_at: now,
                updated_at: now,
            })
            .execute(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "system account {:?} already exists in this workspace",
                        self.system
                    ))
                }
                other => AppError::Internal(format!("system insert failed: {other}")),
            })?;

        Account::from_ref(&account_ref, conn)?
            .ok_or_else(|| AppError::Internal("system account disappeared after insert".into()))
    }
}

// ── Default Account Tree ─────────────────────────────────────────────────────

pub fn create_default_tree(
    workspace: &WorkspaceRef,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    let assets = NewSystemAccount::new(workspace, SystemAccount::AssetsRoot).insert(conn)?;
    let liabilities =
        NewSystemAccount::new(workspace, SystemAccount::LiabilitiesRoot).insert(conn)?;
    let equity = NewSystemAccount::new(workspace, SystemAccount::EquityRoot).insert(conn)?;
    let revenue = NewSystemAccount::new(workspace, SystemAccount::RevenueRoot).insert(conn)?;
    let expenses = NewSystemAccount::new(workspace, SystemAccount::ExpensesRoot).insert(conn)?;

    NewSystemAccount::new(workspace, SystemAccount::Chequing)
        .parent(&assets)
        .insert(conn)?;
    NewSystemAccount::new(workspace, SystemAccount::Savings)
        .parent(&assets)
        .insert(conn)?;
    NewSystemAccount::new(workspace, SystemAccount::RetainedEarnings)
        .parent(&equity)
        .insert(conn)?;
    NewSystemAccount::new(workspace, SystemAccount::OpeningBalances)
        .parent(&equity)
        .insert(conn)?;

    NewCustomAccount::child(&assets, "TFSA").insert(conn)?;
    NewCustomAccount::child(&assets, "RRSP").insert(conn)?;

    NewCustomAccount::child(&liabilities, "Credit Card").insert(conn)?;

    NewCustomAccount::child(&revenue, "Employment").insert(conn)?;

    NewCustomAccount::child(&expenses, "Food").insert(conn)?;
    NewCustomAccount::child(&expenses, "Rent").insert(conn)?;
    NewCustomAccount::child(&expenses, "Groceries").insert(conn)?;
    NewCustomAccount::child(&expenses, "Dining").insert(conn)?;
    NewCustomAccount::child(&expenses, "Transit").insert(conn)?;
    NewCustomAccount::child(&expenses, "Utilities").insert(conn)?;
    NewCustomAccount::child(&expenses, "Entertainment").insert(conn)?;

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

    fn find_account(label: &str, workspace: &Workspace, conn: &mut SqliteConnection) -> Account {
        let all = workspace.as_ref().accounts(conn).unwrap();
        all.into_iter()
            .find(|a| a.name() == label)
            .unwrap_or_else(|| panic!("account not found: {label}"))
    }

    fn find_system(
        system: SystemAccount,
        workspace: &Workspace,
        conn: &mut SqliteConnection,
    ) -> Account {
        workspace
            .as_ref()
            .system_account(system, conn)
            .unwrap()
            .unwrap_or_else(|| panic!("system account not found: {:?}", system))
    }

    #[test]
    fn default_tree_created_on_workspace_creation() {
        let (mut conn, workspace) = setup();

        let roots = workspace.as_ref().root_accounts(&mut conn).unwrap();
        assert_eq!(roots.len(), 5);
        assert!(roots
            .iter()
            .all(|a| matches!(a.account_type, AccountType::System(s) if s.is_root())));

        let all = workspace.as_ref().accounts(&mut conn).unwrap();
        // 5 system roots + 4 system children (Chequing, Savings, RetainedEarnings, OpeningBalances)
        // + 11 custom (TFSA, RRSP, Credit Card, Employment, 7 expenses) = 20
        assert_eq!(all.len(), 20);

        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        assert_eq!(chequing.category(), Category::Asset);
        assert!(matches!(chequing.account_type, AccountType::System(_)));
    }

    #[test]
    fn system_accounts_present() {
        let (mut conn, workspace) = setup();
        for system in [
            SystemAccount::AssetsRoot,
            SystemAccount::LiabilitiesRoot,
            SystemAccount::EquityRoot,
            SystemAccount::RevenueRoot,
            SystemAccount::ExpensesRoot,
            SystemAccount::Chequing,
            SystemAccount::Savings,
            SystemAccount::RetainedEarnings,
            SystemAccount::OpeningBalances,
        ] {
            let account = workspace
                .as_ref()
                .system_account(system, &mut conn)
                .unwrap();
            assert!(account.is_some(), "missing {:?}", system);
        }
    }

    #[test]
    fn duplicate_system_account_rejected() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);
        let err = NewSystemAccount::new(workspace.as_ref(), SystemAccount::Chequing)
            .parent(&assets)
            .insert(&mut conn)
            .expect_err("duplicate system insert should fail");
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn create_custom_account_succeeds() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        let child = NewCustomAccount::child(&assets, "Brokerage")
            .insert(&mut conn)
            .expect("custom creation should succeed");

        assert_eq!(child.name(), "Brokerage");
        assert_eq!(child.category(), Category::Asset);
        assert_eq!(child.parent_id, Some(assets.id.clone()));
        assert!(matches!(child.account_type, AccountType::Custom(_)));
    }

    #[test]
    fn create_deeply_nested_account() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        let dining_out = NewCustomAccount::child(&food, "Dining Out")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(dining_out.category(), Category::Expense);
        assert_eq!(dining_out.parent_id, Some(food.id.clone()));

        assert_eq!(
            dining_out.full_name(&mut conn).unwrap(),
            "Expenses:Food:Dining Out"
        );
    }

    #[test]
    fn sibling_name_uniqueness_enforced() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        // "TFSA" already exists as custom child of Assets
        let err = NewCustomAccount::child(&assets, "TFSA")
            .insert(&mut conn)
            .expect_err("duplicate sibling name should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn cousin_name_duplication_allowed() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        let bank_a = NewCustomAccount::child(&assets, "Bank A")
            .insert(&mut conn)
            .unwrap();
        let bank_b = NewCustomAccount::child(&assets, "Bank B")
            .insert(&mut conn)
            .unwrap();

        NewCustomAccount::child(&bank_a, "Main")
            .insert(&mut conn)
            .unwrap();
        NewCustomAccount::child(&bank_b, "Main")
            .insert(&mut conn)
            .expect("cousin duplication should succeed");
    }

    #[test]
    fn same_system_accounts_different_workspaces() {
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

        let chequing_a = workspace_a
            .as_ref()
            .system_account(SystemAccount::Chequing, &mut conn)
            .unwrap();
        let chequing_b = workspace_b
            .as_ref()
            .system_account(SystemAccount::Chequing, &mut conn)
            .unwrap();
        assert!(chequing_a.is_some());
        assert!(chequing_b.is_some());
        assert_ne!(chequing_a.unwrap().id, chequing_b.unwrap().id);
    }

    #[test]
    fn full_name_returns_colon_delimited_path() {
        let (mut conn, workspace) = setup();
        let expenses = find_system(SystemAccount::ExpensesRoot, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let cafes = NewCustomAccount::child(&food, "Cafés")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(expenses.full_name(&mut conn).unwrap(), "Expenses");
        assert_eq!(food.full_name(&mut conn).unwrap(), "Expenses:Food");
        assert_eq!(cafes.full_name(&mut conn).unwrap(), "Expenses:Food:Cafés");
    }

    #[test]
    fn ancestors_returns_root_to_parent_chain() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);
        let bank = NewCustomAccount::child(&assets, "Bank")
            .insert(&mut conn)
            .unwrap();
        let chequing = NewCustomAccount::child(&bank, "Main Chequing")
            .insert(&mut conn)
            .unwrap();

        let ancestors = chequing.ancestors(&mut conn).unwrap();
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].name(), "Assets");
        assert_eq!(ancestors[1].name(), "Bank");

        let root_ancestors = assets.ancestors(&mut conn).unwrap();
        assert_eq!(root_ancestors.len(), 0);
    }

    #[test]
    fn children_returns_direct_children() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        // Assets default children: Chequing, Savings (System), TFSA, RRSP (Custom) = 4
        let children = assets.children(&mut conn).unwrap();
        assert_eq!(children.len(), 4);
    }

    #[test]
    fn descendants_returns_all_transitive_children() {
        let (mut conn, workspace) = setup();
        let expenses = find_system(SystemAccount::ExpensesRoot, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        NewCustomAccount::child(&food, "Cafés")
            .insert(&mut conn)
            .unwrap();
        NewCustomAccount::child(&food, "Fast Food")
            .insert(&mut conn)
            .unwrap();

        let descendants = expenses.descendants(&mut conn).unwrap();
        // Default expense children: 7. Plus Food's 2 new children = 9
        assert_eq!(descendants.len(), 9);
    }

    #[test]
    fn category_inherited_from_root_via_parent_chain() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);
        let bank = NewCustomAccount::child(&assets, "Bank")
            .insert(&mut conn)
            .unwrap();
        let grandchild = NewCustomAccount::child(&bank, "Premium")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(assets.category(), Category::Asset);
        assert_eq!(bank.category(), Category::Asset);
        assert_eq!(grandchild.category(), Category::Asset);
    }

    #[test]
    fn soft_delete_leaf_succeeds() {
        let (mut conn, workspace) = setup();
        let tfsa = find_account("TFSA", &workspace, &mut conn);

        tfsa.soft_delete(&mut conn).unwrap();

        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);
        let children = assets.children(&mut conn).unwrap();
        assert!(!children.iter().any(|a| a.name() == "TFSA"));
    }

    #[test]
    fn soft_delete_with_active_children_rejected() {
        let (mut conn, workspace) = setup();
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        let err = assets
            .soft_delete(&mut conn)
            .expect_err("delete with active children should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn accounts_returns_only_active() {
        let (mut conn, workspace) = setup();
        let tfsa = find_account("TFSA", &workspace, &mut conn);

        tfsa.soft_delete(&mut conn).unwrap();

        let all = workspace.as_ref().accounts(&mut conn).unwrap();
        assert!(!all.iter().any(|a| a.name() == "TFSA"));
        assert!(all.iter().any(|a| a.name() == "RRSP"));
    }

    #[test]
    fn root_accounts_returns_only_roots() {
        let (mut conn, workspace) = setup();

        let roots = workspace.as_ref().root_accounts(&mut conn).unwrap();
        assert!(roots
            .iter()
            .all(|a| matches!(a.account_type, AccountType::System(s) if s.is_root())));
        assert_eq!(roots.len(), 5);
    }

    // ── Balance tests ───────────────────────────────────────────────────

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
            date,
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
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &food, 800, "2026-01-16", &mut conn);

        assert_eq!(food.balance(&mut conn).unwrap(), 2000);
        assert_eq!(chequing.balance(&mut conn).unwrap(), -2000);
    }

    #[test]
    fn balance_excludes_soft_deleted_entries() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        let deleted_entry = make_entry(&workspace, &chequing, &food, 800, "2026-01-16", &mut conn);
        deleted_entry.soft_delete(&mut conn).unwrap();

        assert_eq!(food.balance(&mut conn).unwrap(), 1200);
    }

    #[test]
    fn balance_with_descendants_includes_children() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let expenses = find_system(SystemAccount::ExpensesRoot, &workspace, &mut conn);

        let cafes = NewCustomAccount::child(&food, "Cafes")
            .insert(&mut conn)
            .unwrap();

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-01-16", &mut conn);

        assert_eq!(food.balance(&mut conn).unwrap(), 1200);

        assert_eq!(
            expenses.balance_with_descendants(&mut conn).unwrap(),
            1700
        );
    }

    #[test]
    fn balance_as_of_filters_by_date() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &food, 800, "2026-01-20", &mut conn);

        assert_eq!(food.balance_as_of("2026-01-15", &mut conn).unwrap(), 1200);
        assert_eq!(food.balance_as_of("2026-01-20", &mut conn).unwrap(), 2000);
        assert_eq!(food.balance_as_of("2026-01-14", &mut conn).unwrap(), 0);
    }

    #[test]
    fn fundamental_equation_holds() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        make_entry(
            &workspace,
            &employment,
            &chequing,
            500000,
            "2026-01-01",
            &mut conn,
        );
        make_entry(&workspace, &chequing, &food, 5000, "2026-01-05", &mut conn);
        make_entry(
            &workspace,
            &chequing,
            &rent,
            150000,
            "2026-01-05",
            &mut conn,
        );

        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);
        let liabilities = find_system(SystemAccount::LiabilitiesRoot, &workspace, &mut conn);
        let equity = find_system(SystemAccount::EquityRoot, &workspace, &mut conn);
        let revenue = find_system(SystemAccount::RevenueRoot, &workspace, &mut conn);
        let expenses = find_system(SystemAccount::ExpensesRoot, &workspace, &mut conn);

        let asset_balance = assets.balance_with_descendants(&mut conn).unwrap();
        let liability_balance = liabilities.balance_with_descendants(&mut conn).unwrap();
        let equity_balance = equity.balance_with_descendants(&mut conn).unwrap();
        let revenue_balance = revenue.balance_with_descendants(&mut conn).unwrap();
        let expense_balance = expenses.balance_with_descendants(&mut conn).unwrap();

        let total = asset_balance
            + liability_balance
            + equity_balance
            + revenue_balance
            + expense_balance;
        assert_eq!(
            total, 0,
            "fundamental equation violated: A={asset_balance} L={liability_balance} E={equity_balance} R={revenue_balance} X={expense_balance}"
        );
    }

    #[test]
    fn account_with_active_postings_cannot_be_deleted() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let err = food
            .soft_delete(&mut conn)
            .expect_err("should fail with active postings");
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn account_with_only_deleted_entry_postings_can_be_deleted() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        entry.soft_delete(&mut conn).unwrap();

        food.soft_delete(&mut conn)
            .expect("should succeed when all entries deleted");
    }

    #[test]
    fn balance_returns_zero_for_no_postings() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);

        assert_eq!(chequing.balance(&mut conn).unwrap(), 0);
    }

    // ── Rename ──────────────────────────────────────────────────────────

    #[test]
    fn rename_custom_succeeds() {
        let (mut conn, workspace) = setup();
        let tfsa = find_account("TFSA", &workspace, &mut conn);

        let renamed = tfsa.rename("TFSA (renamed)", &mut conn).unwrap();

        assert_eq!(renamed.name(), "TFSA (renamed)");
        assert_eq!(renamed.id, tfsa.id);
    }

    #[test]
    fn rename_system_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);

        let result = chequing.rename("Main Chequing", &mut conn);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn rename_duplicate_under_same_parent_rejected() {
        let (mut conn, workspace) = setup();
        let tfsa = find_account("TFSA", &workspace, &mut conn);

        // "RRSP" is a sibling under Assets
        let result = tfsa.rename("RRSP", &mut conn);

        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    // ── Reparent ────────────────────────────────────────────────────────

    #[test]
    fn reparent_succeeds() {
        let (mut conn, workspace) = setup();
        let expenses = find_system(SystemAccount::ExpensesRoot, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let categories = NewCustomAccount::child(&expenses, "Categories")
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
    fn reparent_system_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let liabilities = find_system(SystemAccount::LiabilitiesRoot, &workspace, &mut conn);

        let result = chequing.reparent(&liabilities.id, &mut conn);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_cycle_rejected_non_root() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        let fast_food = NewCustomAccount::child(&food, "Fast Food")
            .insert(&mut conn)
            .unwrap();

        let result = food.reparent(&fast_food.id, &mut conn);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn reparent_cross_category_rejected() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        let result = food.reparent(&assets.id, &mut conn);
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
        let expenses_b = find_system(SystemAccount::ExpensesRoot, &workspace_b, &mut conn);

        let result = food_a.reparent(&expenses_b.id, &mut conn);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    // ── balance_between ────────────────────────────────────────────────

    #[test]
    fn balance_between_returns_sum_in_date_range() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1000, "2026-01-10", &mut conn);
        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &food, 800, "2026-01-20", &mut conn);

        assert_eq!(
            food.balance_between("2026-01-11", "2026-01-19", &mut conn)
                .unwrap(),
            1200
        );
    }

    #[test]
    fn balance_between_inclusive_boundaries() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        assert_eq!(
            food.balance_between("2026-01-15", "2026-01-15", &mut conn)
                .unwrap(),
            1200
        );
    }

    #[test]
    fn balance_between_returns_zero_for_empty_range() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        assert_eq!(
            food.balance_between("2026-02-01", "2026-02-28", &mut conn)
                .unwrap(),
            0
        );
    }

    #[test]
    fn balance_between_with_descendants_includes_children() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let expenses = find_system(SystemAccount::ExpensesRoot, &workspace, &mut conn);

        let cafes = NewCustomAccount::child(&food, "Cafes")
            .insert(&mut conn)
            .unwrap();

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-01-16", &mut conn);

        assert_eq!(
            expenses
                .balance_between_with_descendants("2026-01-01", "2026-01-31", &mut conn)
                .unwrap(),
            1700
        );
    }

    #[test]
    fn balance_as_of_with_descendants_filters_by_date() {
        let (mut conn, workspace) = setup();
        let chequing = find_system(SystemAccount::Chequing, &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let assets = find_system(SystemAccount::AssetsRoot, &workspace, &mut conn);

        let cafes = NewCustomAccount::child(&food, "Cafes")
            .insert(&mut conn)
            .unwrap();

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-02-10", &mut conn);

        assert_eq!(
            assets
                .balance_as_of_with_descendants("2026-01-31", &mut conn)
                .unwrap(),
            -1200
        );
        assert_eq!(
            assets
                .balance_as_of_with_descendants("2026-02-28", &mut conn)
                .unwrap(),
            -1700
        );
    }
}
