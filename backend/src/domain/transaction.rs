use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::account::AccountRef;
use crate::domain::category::CategoryRef;
use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::{line_items, transactions};

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct TransactionRef(String);

impl std::fmt::Display for TransactionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: TransactionRef,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    #[diesel(column_name = account_id)]
    pub account: AccountRef,
    #[diesel(column_name = transfer_account_id)]
    pub transfer_account: Option<AccountRef>,
    pub date: String,
    pub description: String,
    pub amount: i32,
    pub cleared: bool,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Transaction {
    pub fn as_ref(&self) -> &TransactionRef {
        &self.id
    }

    pub fn from_ref(
        transaction_ref: &TransactionRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Transaction>, AppError> {
        transactions::table
            .find(transaction_ref)
            .select(Transaction::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("transaction lookup failed: {err}")))
    }

    pub fn line_items(&self, conn: &mut SqliteConnection) -> Result<Vec<LineItem>, AppError> {
        line_items::table
            .filter(line_items::transaction_id.eq(&self.id))
            .select(LineItem::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("line items query failed: {err}")))
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(transactions::table.find(&self.id))
            .set(transactions::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("transaction soft delete failed: {err}")))?;
        Ok(())
    }

    pub fn cleared_balance_for_account(
        account: &AccountRef,
        conn: &mut SqliteConnection,
    ) -> Result<i64, AppError> {
        transactions::table
            .filter(transactions::account_id.eq(account))
            .filter(transactions::cleared.eq(true))
            .filter(transactions::deleted_at.is_null())
            .select(diesel::dsl::sum(transactions::amount))
            .first::<Option<i64>>(conn)
            .map(|s| s.unwrap_or(0))
            .map_err(|err| AppError::Internal(format!("balance computation failed: {err}")))
    }
}

impl AccountRef {
    pub fn transactions(&self, conn: &mut SqliteConnection) -> Result<Vec<Transaction>, AppError> {
        transactions::table
            .filter(transactions::account_id.eq(self))
            .filter(transactions::deleted_at.is_null())
            .order(transactions::date.desc())
            .select(Transaction::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("transaction list failed: {err}")))
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = line_items)]
pub struct LineItem {
    pub id: String,
    #[diesel(column_name = transaction_id)]
    pub transaction: TransactionRef,
    #[diesel(column_name = category_id)]
    pub category: CategoryRef,
    pub amount: i32,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ── Insertables ──────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = transactions)]
struct NewTransactionRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    #[diesel(column_name = account_id)]
    account: AccountRef,
    #[diesel(column_name = transfer_account_id)]
    transfer_account: Option<AccountRef>,
    date: String,
    description: String,
    amount: i32,
    cleared: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = line_items)]
struct NewLineItemRow {
    id: String,
    #[diesel(column_name = transaction_id)]
    transaction: String,
    #[diesel(column_name = category_id)]
    category: CategoryRef,
    amount: i32,
    note: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

// ── NewTransaction ───────────────────────────────────────────────────────────

pub struct LineItemInput {
    pub category: CategoryRef,
    pub amount: i32,
    pub note: Option<String>,
}

impl LineItemInput {
    pub fn new(category: &CategoryRef, amount: i32) -> Self {
        LineItemInput {
            category: category.clone(),
            amount,
            note: None,
        }
    }

    pub fn note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
}

pub struct NewTransaction {
    workspace: WorkspaceRef,
    account: AccountRef,
    transfer_account: Option<AccountRef>,
    date: String,
    description: String,
    amount: i32,
    cleared: bool,
    line_items: Vec<LineItemInput>,
}

impl NewTransaction {
    pub fn new(
        workspace: &WorkspaceRef,
        account: &AccountRef,
        date: &str,
        amount: i32,
        line_items: Vec<LineItemInput>,
    ) -> Self {
        NewTransaction {
            workspace: workspace.clone(),
            account: account.clone(),
            transfer_account: None,
            date: date.to_string(),
            description: String::new(),
            amount,
            cleared: false,
            line_items,
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn cleared(mut self, cleared: bool) -> Self {
        self.cleared = cleared;
        self
    }

    pub fn transfer_to(mut self, account: &AccountRef) -> Self {
        self.transfer_account = Some(account.clone());
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Transaction, AppError> {
        if self.line_items.is_empty() {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "line_items".into(),
                message: "at least one line item is required".into(),
            }]));
        }

        if let Some(li) = self.line_items.iter().find(|li| li.amount == 0) {
            let _ = li;
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "line_items".into(),
                message: "line item amount must be non-zero".into(),
            }]));
        }

        let li_sum: i32 = self.line_items.iter().map(|li| li.amount).sum();
        if li_sum != self.amount {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "line_items".into(),
                message: format!(
                    "line items sum ({li_sum}) does not match transaction amount ({})",
                    self.amount
                ),
            }]));
        }

        conn.transaction::<_, AppError, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();
            let transaction_id = Uuid::new_v4().to_string();

            let transaction = diesel::insert_into(transactions::table)
                .values(NewTransactionRow {
                    id: transaction_id.clone(),
                    workspace: self.workspace,
                    account: self.account,
                    transfer_account: self.transfer_account,
                    date: self.date,
                    description: self.description,
                    amount: self.amount,
                    cleared: self.cleared,
                    created_at: now,
                    updated_at: now,
                })
                .returning(Transaction::as_returning())
                .get_result(conn)?;

            for li in self.line_items {
                diesel::insert_into(line_items::table)
                    .values(NewLineItemRow {
                        id: Uuid::new_v4().to_string(),
                        transaction: transaction_id.clone(),
                        category: li.category,
                        amount: li.amount,
                        note: li.note,
                        created_at: now,
                        updated_at: now,
                    })
                    .execute(conn)?;
            }

            Ok(transaction)
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{Account, AccountType, NewAccount};
    use crate::domain::category::{Category, NewCategory};
    use crate::domain::user::NewUser;
    use crate::domain::workspace::{Workspace,NewWorkspace};
    use crate::test_helpers::setup_test_db;

    struct TestFixture {
        conn: SqliteConnection,
        workspace: Workspace,
        chequing: Account,
        groceries: Category,
    }

    fn setup() -> TestFixture {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace = NewWorkspace::new("My Finances", user.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        let chequing = NewAccount::new(&workspace.as_ref(), "Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .unwrap();
        let groceries = NewCategory::new(&workspace.as_ref(), "Groceries")
            .insert(&mut conn)
            .unwrap();
        TestFixture {
            conn,
            workspace,
            chequing,
            groceries,
        }
    }

    #[test]
    fn create_transaction_with_single_line_item() {
        let mut fixture = setup();
        let transaction = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            -5000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -5000)],
        )
        .description("Weekly groceries")
        .insert(&mut fixture.conn)
        .expect("transaction creation should succeed");

        assert_eq!(transaction.amount, -5000);
        assert_eq!(transaction.description, "Weekly groceries");
        assert!(!transaction.cleared);
    }

    #[test]
    fn create_transaction_with_multiple_line_items() {
        let mut fixture = setup();
        let dining = NewCategory::new(&fixture.workspace.as_ref(), "Dining")
            .insert(&mut fixture.conn)
            .unwrap();

        let transaction = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            -7500,
            vec![
                LineItemInput::new(&fixture.groceries.as_ref(), -5000),
                LineItemInput::new(&dining.as_ref(), -2500),
            ],
        )
        .insert(&mut fixture.conn)
        .expect("multi-line transaction should succeed");

        let items = transaction.line_items(&mut fixture.conn).unwrap();
        assert_eq!(items.len(), 2);
        let sum: i32 = items.iter().map(|li| li.amount).sum();
        assert_eq!(sum, transaction.amount);
    }

    #[test]
    fn line_items_must_sum_to_total() {
        let mut fixture = setup();
        let err = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            -5000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -3000)],
        )
        .insert(&mut fixture.conn)
        .expect_err("mismatched sum should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn at_least_one_line_item_required() {
        let mut fixture = setup();
        let err = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            -5000,
            vec![],
        )
        .insert(&mut fixture.conn)
        .expect_err("empty line items should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn zero_amount_line_item_rejected() {
        let mut fixture = setup();
        let err = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            0,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), 0)],
        )
        .insert(&mut fixture.conn)
        .expect_err("zero amount should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn cleared_balance_computation() {
        let mut fixture = setup();
        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            100000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), 100000)],
        )
        .cleared(true)
        .description("Paycheck")
        .insert(&mut fixture.conn)
        .unwrap();

        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-12",
            -5000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -5000)],
        )
        .cleared(true)
        .insert(&mut fixture.conn)
        .unwrap();

        // Pending — should not affect cleared balance
        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-14",
            -2000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -2000)],
        )
        .insert(&mut fixture.conn)
        .unwrap();

        let balance =
            Transaction::cleared_balance_for_account(&fixture.chequing.as_ref(), &mut fixture.conn)
                .unwrap();
        assert_eq!(balance, 95000); // 100000 - 5000
    }

    #[test]
    fn soft_deleted_transactions_excluded_from_balance() {
        let mut fixture = setup();
        let transaction = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            100000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), 100000)],
        )
        .cleared(true)
        .insert(&mut fixture.conn)
        .unwrap();

        transaction.soft_delete(&mut fixture.conn).unwrap();

        let balance =
            Transaction::cleared_balance_for_account(&fixture.chequing.as_ref(), &mut fixture.conn)
                .unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn transactions_returns_only_active() {
        let mut fixture = setup();
        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            -5000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -5000)],
        )
        .insert(&mut fixture.conn)
        .unwrap();

        let deleted = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-11",
            -3000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -3000)],
        )
        .insert(&mut fixture.conn)
        .unwrap();
        deleted.soft_delete(&mut fixture.conn).unwrap();

        let transactions = fixture
            .chequing
            .as_ref()
            .transactions(&mut fixture.conn)
            .unwrap();
        assert_eq!(transactions.len(), 1);
    }

    #[test]
    fn transfer_transaction() {
        let mut fixture = setup();
        let savings = NewAccount::new(&fixture.workspace.as_ref(), "Savings", AccountType::Savings)
            .insert(&mut fixture.conn)
            .unwrap();

        let transaction = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            -10000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -10000)],
        )
        .transfer_to(&savings.as_ref())
        .insert(&mut fixture.conn)
        .unwrap();

        assert_eq!(transaction.transfer_account, Some(savings.as_ref().clone()));
    }

    #[test]
    fn line_item_note() {
        let mut fixture = setup();
        let transaction = NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-15",
            -5000,
            vec![LineItemInput::new(&fixture.groceries.as_ref(), -5000).note("Costco run")],
        )
        .insert(&mut fixture.conn)
        .unwrap();

        let items = transaction.line_items(&mut fixture.conn).unwrap();
        assert_eq!(items[0].note, Some("Costco run".to_string()));
    }
}
