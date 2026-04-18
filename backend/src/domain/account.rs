use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use uuid::Uuid;

use crate::domain::workspace::Workspace;
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct AccountRef(String);


#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = accounts)]
pub struct Account {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Account {
    pub fn as_ref(&self) -> AccountRef {
        AccountRef(self.id.clone())
    }

    pub fn from_ref(account_ref: &AccountRef, conn: &mut SqliteConnection) -> Result<Option<Account>, AppError> {
        accounts::table
            .find(&account_ref.0)
            .select(Account::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("account lookup failed: {err}")))
    }

    pub fn for_workspace(
        workspace: &Workspace,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Account>, AppError> {
        accounts::table
            .filter(accounts::workspace_id.eq(&workspace.id))
            .filter(accounts::deleted_at.is_null())
            .select(Account::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("account list failed: {err}")))
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

// ── NewAccount ───────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = accounts)]
struct NewAccountRow {
    id: String,
    workspace_id: String,
    name: String,
    account_type: String,
    currency: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewAccount {
    workspace_id: String,
    name: String,
    account_type: AccountType,
    currency: String,
}

impl NewAccount {
    pub fn new(workspace: &Workspace, name: &str, account_type: AccountType) -> Self {
        NewAccount {
            workspace_id: workspace.id.clone(),
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
                workspace_id: self.workspace_id,
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
                    AppError::Conflict(format!("account '{}' already exists in this workspace", self.name))
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
    use crate::domain::workspace::NewWorkspace;
    use crate::test_helpers::setup_test_db;

    fn setup() -> (SqliteConnection, Workspace) {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap().insert(&mut conn).unwrap();
        let ws = NewWorkspace::new("My Finances", &user)
            .insert(&mut conn).unwrap();
        (conn, ws)
    }

    #[test]
    fn create_account_succeeds() {
        let (mut conn, ws) = setup();
        let acct = NewAccount::new(&ws, "Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .expect("account creation should succeed");

        assert_eq!(acct.name, "Chequing");
        assert_eq!(acct.account_type, "chequing");
        assert_eq!(acct.currency, "CAD");
        assert_eq!(acct.workspace_id, ws.id);
    }

    #[test]
    fn custom_currency() {
        let (mut conn, ws) = setup();
        let acct = NewAccount::new(&ws, "USD Savings", AccountType::Savings)
            .currency("USD")
            .insert(&mut conn)
            .unwrap();

        assert_eq!(acct.currency, "USD");
    }

    #[test]
    fn duplicate_name_in_same_workspace_fails() {
        let (mut conn, ws) = setup();
        NewAccount::new(&ws, "Chequing", AccountType::Chequing)
            .insert(&mut conn).unwrap();

        let err = NewAccount::new(&ws, "Chequing", AccountType::Savings)
            .insert(&mut conn)
            .expect_err("duplicate name should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_type_different_names_succeeds() {
        let (mut conn, ws) = setup();
        NewAccount::new(&ws, "TFSA 1", AccountType::Tfsa)
            .insert(&mut conn).unwrap();
        NewAccount::new(&ws, "TFSA 2", AccountType::Tfsa)
            .insert(&mut conn).unwrap();
    }

    #[test]
    fn for_workspace_returns_only_active_accounts() {
        let (mut conn, ws) = setup();
        NewAccount::new(&ws, "Chequing", AccountType::Chequing)
            .insert(&mut conn).unwrap();
        let savings = NewAccount::new(&ws, "Savings", AccountType::Savings)
            .insert(&mut conn).unwrap();
        savings.soft_delete(&mut conn).unwrap();

        let accts = Account::for_workspace(&ws, &mut conn).unwrap();
        assert_eq!(accts.len(), 1);
        assert_eq!(accts[0].name, "Chequing");
    }

    #[test]
    fn for_workspace_excludes_other_workspaces() {
        let mut conn = setup_test_db();
        let user_a = NewUser::new("a@example.com", "A", "password").unwrap()
            .insert(&mut conn).unwrap();
        let user_b = NewUser::new("b@example.com", "B", "password").unwrap()
            .insert(&mut conn).unwrap();
        let ws_a = NewWorkspace::new("WS A", &user_a).insert(&mut conn).unwrap();
        let ws_b = NewWorkspace::new("WS B", &user_b).insert(&mut conn).unwrap();

        NewAccount::new(&ws_a, "A's Chequing", AccountType::Chequing)
            .insert(&mut conn).unwrap();
        NewAccount::new(&ws_b, "B's Chequing", AccountType::Chequing)
            .insert(&mut conn).unwrap();

        let accts = Account::for_workspace(&ws_a, &mut conn).unwrap();
        assert_eq!(accts.len(), 1);
        assert_eq!(accts[0].name, "A's Chequing");
    }

    #[test]
    fn parsed_type_roundtrips() {
        let (mut conn, ws) = setup();
        let acct = NewAccount::new(&ws, "My RRSP", AccountType::Rrsp)
            .insert(&mut conn).unwrap();

        assert_eq!(acct.parsed_type(), Some(AccountType::Rrsp));
    }
}
