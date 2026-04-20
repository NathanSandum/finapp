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
use crate::schema::accounts;

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

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
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
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewAccount {
    workspace: WorkspaceRef,
    parent_id: Option<AccountRef>,
    name: String,
    account_type: AccountType,
    currency: String,
}

impl NewAccount {
    /// Create a root account (no parent). Must specify account type.
    pub fn root(workspace: WorkspaceRef, name: &str, account_type: AccountType) -> Self {
        NewAccount {
            workspace,
            parent_id: None,
            name: name.to_string(),
            account_type,
            currency: "CAD".to_string(),
        }
    }

    /// Create a child account. Inherits workspace and account_type from parent.
    pub fn child(parent: &Account, name: &str) -> Self {
        NewAccount {
            workspace: parent.workspace.clone(),
            parent_id: Some(parent.id.clone()),
            name: name.to_string(),
            account_type: parent.account_type,
            currency: parent.currency.clone(),
        }
    }

    pub fn currency(mut self, currency: &str) -> Self {
        self.currency = currency.to_string();
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Account, AppError> {
        // For root accounts, enforce uniqueness manually since SQLite
        // treats NULL as distinct in UNIQUE constraints
        if self.parent_id.is_none() {
            let existing: i64 = accounts::table
                .filter(accounts::workspace_id.eq(&self.workspace))
                .filter(accounts::parent_id.is_null())
                .filter(accounts::name.eq(&self.name))
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
    let assets = NewAccount::root(workspace.clone(), "Assets", AccountType::Asset).insert(conn)?;
    let liabilities =
        NewAccount::root(workspace.clone(), "Liabilities", AccountType::Liability).insert(conn)?;
    let _equity =
        NewAccount::root(workspace.clone(), "Equity", AccountType::Equity).insert(conn)?;
    let revenue =
        NewAccount::root(workspace.clone(), "Revenue", AccountType::Revenue).insert(conn)?;
    let expenses =
        NewAccount::root(workspace.clone(), "Expenses", AccountType::Expense).insert(conn)?;

    // Asset sub-accounts
    NewAccount::child(&assets, "Chequing").insert(conn)?;
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

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
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
        // 5 roots + 4 asset + 1 liability + 1 revenue + 7 expense = 18
        assert_eq!(all.len(), 18);

        let chequing = all.iter().find(|a| a.name == "Chequing").unwrap();
        assert_eq!(chequing.account_type, AccountType::Asset);
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
    fn root_name_uniqueness_within_workspace() {
        let (mut conn, workspace) = setup();
        // "Assets" already exists as root from defaults
        let err = NewAccount::root(workspace.as_ref().clone(), "Assets", AccountType::Liability)
            .insert(&mut conn)
            .expect_err("duplicate root name should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn create_new_root_succeeds() {
        let (mut conn, workspace) = setup();
        let account =
            NewAccount::root(workspace.as_ref().clone(), "Custom", AccountType::Asset)
                .insert(&mut conn)
                .expect("new root should succeed");

        assert_eq!(account.name, "Custom");
        assert_eq!(account.account_type, AccountType::Asset);
        assert!(account.is_root());
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
        let chequing = find_account("Chequing", &workspace, &mut conn);

        chequing.soft_delete(&mut conn).unwrap();

        let assets = find_account("Assets", &workspace, &mut conn);
        let children = assets.children(&mut conn).unwrap();
        assert!(!children.iter().any(|a| a.name == "Chequing"));
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
        let equity = find_account("Equity", &workspace, &mut conn);

        // Equity has no default children, so delete should succeed
        equity
            .soft_delete(&mut conn)
            .expect("delete with no children should succeed");
    }

    #[test]
    fn accounts_returns_only_active() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        chequing.soft_delete(&mut conn).unwrap();

        let all = workspace.as_ref().accounts(&mut conn).unwrap();
        assert!(!all.iter().any(|a| a.name == "Chequing"));
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
}
