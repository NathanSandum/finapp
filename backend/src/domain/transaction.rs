use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::account::Account;
use crate::domain::category::Category;
use crate::domain::workspace::Workspace;
use crate::error::AppError;
use crate::schema::{line_items, transactions};

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct TransactionRef(String);


#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: String,
    #[diesel(column_name = workspace_id)]
    pub workspace: String,
    #[diesel(column_name = account_id)]
    pub account: String,
    #[diesel(column_name = transfer_account_id)]
    pub transfer_account: Option<String>,
    pub date: String,
    pub description: String,
    pub amount: i32,
    pub cleared: bool,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Transaction {
    pub fn as_ref(&self) -> TransactionRef {
        TransactionRef(self.id.clone())
    }

    pub fn from_ref(transaction_ref: &TransactionRef, conn: &mut SqliteConnection) -> Result<Option<Transaction>, AppError> {
        transactions::table
            .find(&transaction_ref.0)
            .select(Transaction::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("transaction lookup failed: {err}")))
    }

    pub fn for_account(
        account: &Account,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Transaction>, AppError> {
        transactions::table
            .filter(transactions::account_id.eq(&account.id))
            .filter(transactions::deleted_at.is_null())
            .order(transactions::date.desc())
            .select(Transaction::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("transaction list failed: {err}")))
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
        account: &Account,
        conn: &mut SqliteConnection,
    ) -> Result<i64, AppError> {
        transactions::table
            .filter(transactions::account_id.eq(&account.id))
            .filter(transactions::cleared.eq(true))
            .filter(transactions::deleted_at.is_null())
            .select(diesel::dsl::sum(transactions::amount))
            .first::<Option<i64>>(conn)
            .map(|s| s.unwrap_or(0))
            .map_err(|err| AppError::Internal(format!("balance computation failed: {err}")))
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = line_items)]
pub struct LineItem {
    pub id: String,
    #[diesel(column_name = transaction_id)]
    pub transaction: String,
    #[diesel(column_name = category_id)]
    pub category: String,
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
    workspace: String,
    #[diesel(column_name = account_id)]
    account: String,
    #[diesel(column_name = transfer_account_id)]
    transfer_account: Option<String>,
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
    category: String,
    amount: i32,
    note: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

// ── NewTransaction ───────────────────────────────────────────────────────────

pub struct LineItemInput {
    pub category: String,
    pub amount: i32,
    pub note: Option<String>,
}

impl LineItemInput {
    pub fn new(cat: &Category, amount: i32) -> Self {
        LineItemInput {
            category: cat.id.clone(),
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
    workspace: String,
    account: String,
    transfer_account: Option<String>,
    date: String,
    description: String,
    amount: i32,
    cleared: bool,
    line_items: Vec<LineItemInput>,
}

impl NewTransaction {
    pub fn new(
        workspace: &Workspace,
        account: &Account,
        date: &str,
        amount: i32,
        line_items: Vec<LineItemInput>,
    ) -> Self {
        NewTransaction {
            workspace: workspace.id.clone(),
            account: account.id.clone(),
            transfer_account: None,
            date: date.to_string(),
            description: String::new(),
            amount,
            cleared: false,
            line_items,
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn cleared(mut self, cleared: bool) -> Self {
        self.cleared = cleared;
        self
    }

    pub fn transfer_to(mut self, account: &Account) -> Self {
        self.transfer_account = Some(account.id.clone());
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Transaction, AppError> {
        // Validate: at least one line item
        if self.line_items.is_empty() {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "line_items".into(),
                message: "at least one line item is required".into(),
            }]));
        }

        // Validate: no zero-amount line items
        if let Some(li) = self.line_items.iter().find(|li| li.amount == 0) {
            let _ = li;
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "line_items".into(),
                message: "line item amount must be non-zero".into(),
            }]));
        }

        // Validate: line items sum to transaction total
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
            let txn_id = Uuid::new_v4().to_string();

            let txn = diesel::insert_into(transactions::table)
                .values(NewTransactionRow {
                    id: txn_id.clone(),
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
                        transaction: txn_id.clone(),
                        category: li.category,
                        amount: li.amount,
                        note: li.note,
                        created_at: now,
                        updated_at: now,
                    })
                    .execute(conn)?;
            }

            Ok(txn)
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{AccountType, NewAccount};
    use crate::domain::category::NewCategory;
    use crate::domain::user::NewUser;
    use crate::domain::workspace::NewWorkspace;
    use crate::test_helpers::setup_test_db;

    struct TestFixture {
        conn: SqliteConnection,
        ws: Workspace,
        chequing: Account,
        groceries: Category,
    }

    fn setup() -> TestFixture {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap().insert(&mut conn).unwrap();
        let ws = NewWorkspace::new("My Finances", &user)
            .insert(&mut conn).unwrap();
        let chequing = NewAccount::new(&ws, "Chequing", AccountType::Chequing)
            .insert(&mut conn).unwrap();
        let groceries = NewCategory::new(&ws, "Groceries")
            .insert(&mut conn).unwrap();
        TestFixture { conn, ws, chequing, groceries }
    }

    #[test]
    fn create_transaction_with_single_line_item() {
        let mut f = setup();
        let txn = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", -5000,
            vec![LineItemInput::new(&f.groceries, -5000)],
        )
            .description("Weekly groceries")
            .insert(&mut f.conn)
            .expect("transaction creation should succeed");

        assert_eq!(txn.amount, -5000);
        assert_eq!(txn.description, "Weekly groceries");
        assert_eq!(txn.date, "2026-04-15");
        assert!(!txn.cleared);
    }

    #[test]
    fn create_transaction_with_multiple_line_items() {
        let mut f = setup();
        let dining = NewCategory::new(&f.ws, "Dining")
            .insert(&mut f.conn).unwrap();

        let txn = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", -7500,
            vec![
                LineItemInput::new(&f.groceries, -5000),
                LineItemInput::new(&dining, -2500),
            ],
        )
            .insert(&mut f.conn)
            .expect("multi-line transaction should succeed");

        let items = txn.line_items(&mut f.conn).unwrap();
        assert_eq!(items.len(), 2);
        let sum: i32 = items.iter().map(|li| li.amount).sum();
        assert_eq!(sum, txn.amount);
    }

    #[test]
    fn line_items_must_sum_to_total() {
        let mut f = setup();
        let err = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", -5000,
            vec![LineItemInput::new(&f.groceries, -3000)],
        )
            .insert(&mut f.conn)
            .expect_err("mismatched sum should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn at_least_one_line_item_required() {
        let mut f = setup();
        let err = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", -5000,
            vec![],
        )
            .insert(&mut f.conn)
            .expect_err("empty line items should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn zero_amount_line_item_rejected() {
        let mut f = setup();
        let err = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", 0,
            vec![LineItemInput::new(&f.groceries, 0)],
        )
            .insert(&mut f.conn)
            .expect_err("zero amount should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn cleared_balance_computation() {
        let mut f = setup();
        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", 100000,
            vec![LineItemInput::new(&f.groceries, 100000)],
        )
            .cleared(true)
            .description("Paycheck")
            .insert(&mut f.conn).unwrap();

        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-12", -5000,
            vec![LineItemInput::new(&f.groceries, -5000)],
        )
            .cleared(true)
            .insert(&mut f.conn).unwrap();

        // Pending — should not affect cleared balance
        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-14", -2000,
            vec![LineItemInput::new(&f.groceries, -2000)],
        )
            .insert(&mut f.conn).unwrap();

        let balance = Transaction::cleared_balance_for_account(&f.chequing, &mut f.conn).unwrap();
        assert_eq!(balance, 95000); // 100000 - 5000
    }

    #[test]
    fn soft_deleted_transactions_excluded_from_balance() {
        let mut f = setup();
        let txn = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", 100000,
            vec![LineItemInput::new(&f.groceries, 100000)],
        )
            .cleared(true)
            .insert(&mut f.conn).unwrap();

        txn.soft_delete(&mut f.conn).unwrap();

        let balance = Transaction::cleared_balance_for_account(&f.chequing, &mut f.conn).unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn for_account_returns_only_active_transactions() {
        let mut f = setup();
        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", -5000,
            vec![LineItemInput::new(&f.groceries, -5000)],
        )
            .insert(&mut f.conn).unwrap();

        let deleted = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-11", -3000,
            vec![LineItemInput::new(&f.groceries, -3000)],
        )
            .insert(&mut f.conn).unwrap();
        deleted.soft_delete(&mut f.conn).unwrap();

        let txns = Transaction::for_account(&f.chequing, &mut f.conn).unwrap();
        assert_eq!(txns.len(), 1);
    }

    #[test]
    fn transfer_transaction() {
        let mut f = setup();
        let savings = NewAccount::new(&f.ws, "Savings", AccountType::Savings)
            .insert(&mut f.conn).unwrap();

        let txn = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", -10000,
            vec![LineItemInput::new(&f.groceries, -10000)],
        )
            .transfer_to(&savings)
            .insert(&mut f.conn)
            .unwrap();

        assert_eq!(txn.transfer_account, Some(savings.id));
    }

    #[test]
    fn line_item_note() {
        let mut f = setup();
        let txn = NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-15", -5000,
            vec![LineItemInput::new(&f.groceries, -5000).note("Costco run")],
        )
            .insert(&mut f.conn).unwrap();

        let items = txn.line_items(&mut f.conn).unwrap();
        assert_eq!(items[0].note, Some("Costco run".to_string()));
    }
}
