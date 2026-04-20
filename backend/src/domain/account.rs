use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::accounts;

// ── AccountType ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    Chequing,
    Savings,
    Tfsa,
    Rrsp,
    Brokerage,
    Credit,
    Other,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Chequing => "chequing",
            AccountType::Savings => "savings",
            AccountType::Tfsa => "tfsa",
            AccountType::Rrsp => "rrsp",
            AccountType::Brokerage => "brokerage",
            AccountType::Credit => "credit",
            AccountType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "chequing" => Some(AccountType::Chequing),
            "savings" => Some(AccountType::Savings),
            "tfsa" => Some(AccountType::Tfsa),
            "rrsp" => Some(AccountType::Rrsp),
            "brokerage" => Some(AccountType::Brokerage),
            "credit" => Some(AccountType::Credit),
            "other" => Some(AccountType::Other),
            _ => None,
        }
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
    pub name: String,
    pub account_type: String,
    pub currency: String,
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

    pub fn parsed_type(&self) -> Option<AccountType> {
        AccountType::from_str(&self.account_type)
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(accounts::table.find(&self.id))
            .set(accounts::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("account soft delete failed: {err}")))?;
        Ok(())
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
}

// ── NewAccount ───────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = accounts)]
struct NewAccountRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    name: String,
    account_type: String,
    currency: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewAccount {
    workspace: WorkspaceRef,
    name: String,
    account_type: AccountType,
    currency: String,
}

impl NewAccount {
    pub fn new(workspace: &WorkspaceRef, name: &str, account_type: AccountType) -> Self {
        NewAccount {
            workspace: workspace.clone(),
            name: name.to_string(),
            account_type,
            currency: "CAD".to_string(),
        }
    }

    pub fn currency(mut self, currency: &str) -> Self {
        self.currency = currency.to_string();
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Account, AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::insert_into(accounts::table)
            .values(NewAccountRow {
                id: Uuid::new_v4().to_string(),
                workspace: self.workspace,
                name: self.name.clone(),
                account_type: self.account_type.as_str().to_string(),
                currency: self.currency,
                created_at: now,
                updated_at: now,
            })
            .returning(Account::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "account '{}' already exists in this workspace",
                        self.name
                    ))
                }
                other => AppError::Internal(format!("account insert failed: {other}")),
            })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn create_account_succeeds() {
        let (mut conn, workspace) = setup();
        let account = NewAccount::new(&workspace.as_ref(), "Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .expect("account creation should succeed");

        assert_eq!(account.name, "Chequing");
        assert_eq!(account.account_type, "chequing");
        assert_eq!(account.currency, "CAD");
        assert_eq!(account.workspace, workspace.id);
    }

    #[test]
    fn custom_currency() {
        let (mut conn, workspace) = setup();
        let account = NewAccount::new(&workspace.as_ref(), "USD Savings", AccountType::Savings)
            .currency("USD")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(account.currency, "USD");
    }

    #[test]
    fn duplicate_name_in_same_workspace_fails() {
        let (mut conn, workspace) = setup();
        NewAccount::new(&workspace.as_ref(), "Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .unwrap();

        let err = NewAccount::new(&workspace.as_ref(), "Chequing", AccountType::Savings)
            .insert(&mut conn)
            .expect_err("duplicate name should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_type_different_names_succeeds() {
        let (mut conn, workspace) = setup();
        NewAccount::new(&workspace.as_ref(), "TFSA 1", AccountType::Tfsa)
            .insert(&mut conn)
            .unwrap();
        NewAccount::new(&workspace.as_ref(), "TFSA 2", AccountType::Tfsa)
            .insert(&mut conn)
            .unwrap();
    }

    #[test]
    fn accounts_returns_only_active() {
        let (mut conn, workspace) = setup();
        NewAccount::new(&workspace.as_ref(), "Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .unwrap();
        let savings = NewAccount::new(&workspace.as_ref(), "Savings", AccountType::Savings)
            .insert(&mut conn)
            .unwrap();
        savings.soft_delete(&mut conn).unwrap();

        let accounts = workspace.as_ref().accounts(&mut conn).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Chequing");
    }

    #[test]
    fn accounts_excludes_other_workspaces() {
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

        NewAccount::new(&workspace_a.as_ref(), "A's Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .unwrap();
        NewAccount::new(&workspace_b.as_ref(), "B's Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .unwrap();

        let accounts = workspace_a.as_ref().accounts(&mut conn).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "A's Chequing");
    }

    #[test]
    fn parsed_type_roundtrips() {
        let (mut conn, workspace) = setup();
        let account = NewAccount::new(&workspace.as_ref(), "My RRSP", AccountType::Rrsp)
            .insert(&mut conn)
            .unwrap();

        assert_eq!(account.parsed_type(), Some(AccountType::Rrsp));
    }
}
