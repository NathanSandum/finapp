use chrono::NaiveDateTime;
use diesel::prelude::*;

use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::account::AccountRef;
use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::{postings, transactions};

// ── TransactionRef ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct TransactionRef(String);

impl std::fmt::Display for TransactionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── PostingRef ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct PostingRef(String);

impl std::fmt::Display for PostingRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Transaction ─────────────────────────────────────────────────────────────

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Transaction {
    pub id: TransactionRef,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    pub date: String,
    pub description: String,
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

    pub fn postings(&self, conn: &mut SqliteConnection) -> Result<Vec<Posting>, AppError> {
        postings::table
            .filter(postings::transaction_id.eq(&self.id))
            .select(Posting::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("postings query failed: {err}")))
    }

    /// Transactions with any posting to this account, excluding soft-deleted, ordered by date desc.
    pub fn for_account(
        account: &AccountRef,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Transaction>, AppError> {
        transactions::table
            .inner_join(postings::table.on(postings::transaction_id.eq(transactions::id)))
            .filter(postings::account_id.eq(account))
            .filter(transactions::deleted_at.is_null())
            .select(Transaction::as_select())
            .distinct()
            .order(transactions::date.desc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("for_account query failed: {err}")))
    }

    /// Transactions with postings to this account or any of its descendants.
    pub fn for_account_tree(
        account: &crate::domain::account::Account,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Transaction>, AppError> {
        let descendants = account.descendants(conn)?;
        let mut account_ids: Vec<&AccountRef> = descendants.iter().map(|a| &a.id).collect();
        account_ids.push(&account.id);

        transactions::table
            .inner_join(postings::table.on(postings::transaction_id.eq(transactions::id)))
            .filter(postings::account_id.eq_any(&account_ids))
            .filter(transactions::deleted_at.is_null())
            .select(Transaction::as_select())
            .distinct()
            .order(transactions::date.desc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("for_account_tree query failed: {err}")))
    }

    pub fn for_workspace(
        workspace: &WorkspaceRef,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Transaction>, AppError> {
        transactions::table
            .filter(transactions::workspace_id.eq(workspace))
            .filter(transactions::deleted_at.is_null())
            .select(Transaction::as_select())
            .order(transactions::date.desc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("for_workspace query failed: {err}")))
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(transactions::table.find(&self.id))
            .set(transactions::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("transaction soft delete failed: {err}")))?;
        Ok(())
    }
}

// ── Posting ─────────────────────────────────────────────────────────────────

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = postings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Posting {
    pub id: PostingRef,
    #[diesel(column_name = transaction_id)]
    pub transaction: TransactionRef,
    #[diesel(column_name = account_id)]
    pub account: AccountRef,
    pub amount: i32,
    pub note: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Posting {
    pub fn from_ref(
        posting_ref: &PostingRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Posting>, AppError> {
        postings::table
            .find(posting_ref)
            .select(Posting::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("posting lookup failed: {err}")))
    }
}

// ── NewTransaction ──────────────────────────────────────────────────────────

pub struct PostingInput {
    pub account: AccountRef,
    pub amount: i32,
    pub note: Option<String>,
}

impl PostingInput {
    pub fn new(account: AccountRef, amount: i32) -> Self {
        PostingInput {
            account,
            amount,
            note: None,
        }
    }

    pub fn note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
}

#[derive(Insertable)]
#[diesel(table_name = transactions)]
struct NewTransactionRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    date: String,
    description: String,
    cleared: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = postings)]
struct NewPostingRow {
    id: String,
    #[diesel(column_name = transaction_id)]
    transaction: String,
    #[diesel(column_name = account_id)]
    account: AccountRef,
    amount: i32,
    note: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewTransaction {
    workspace: WorkspaceRef,
    date: String,
    description: String,
    cleared: bool,
    posting_inputs: Vec<PostingInput>,
}

impl NewTransaction {
    pub fn new(
        workspace: WorkspaceRef,
        date: &str,
        description: &str,
        posting_inputs: Vec<PostingInput>,
    ) -> Self {
        NewTransaction {
            workspace,
            date: date.to_string(),
            description: description.to_string(),
            cleared: false,
            posting_inputs,
        }
    }

    pub fn cleared(mut self, cleared: bool) -> Self {
        self.cleared = cleared;
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Transaction, AppError> {
        // Validate: at least 2 postings
        if self.posting_inputs.len() < 2 {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "postings".into(),
                message: "transaction must have at least 2 postings".into(),
            }]));
        }

        // Validate: no zero amounts
        for input in &self.posting_inputs {
            if input.amount == 0 {
                return Err(AppError::Validation(vec![crate::error::FieldError {
                    field: "postings".into(),
                    message: "posting amount cannot be zero".into(),
                }]));
            }
        }

        // Validate: sum to zero
        let sum: i64 = self.posting_inputs.iter().map(|p| p.amount as i64).sum();
        if sum != 0 {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "postings".into(),
                message: format!("postings must sum to zero, got {sum}"),
            }]));
        }

        conn.transaction::<_, AppError, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();
            let transaction_id = Uuid::new_v4().to_string();

            let transaction = diesel::insert_into(transactions::table)
                .values(NewTransactionRow {
                    id: transaction_id.clone(),
                    workspace: self.workspace,
                    date: self.date,
                    description: self.description,
                    cleared: self.cleared,
                    created_at: now,
                    updated_at: now,
                })
                .returning(Transaction::as_returning())
                .get_result(conn)?;

            for input in self.posting_inputs {
                diesel::insert_into(postings::table)
                    .values(NewPostingRow {
                        id: Uuid::new_v4().to_string(),
                        transaction: transaction_id.clone(),
                        account: input.account,
                        amount: input.amount,
                        note: input.note.unwrap_or_default(),
                        created_at: now,
                        updated_at: now,
                    })
                    .execute(conn)?;
            }

            Ok(transaction)
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{Account, NewAccount};
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

    fn find_account(name: &str, workspace: &Workspace, conn: &mut SqliteConnection) -> Account {
        let all = workspace.as_ref().accounts(conn).unwrap();
        all.into_iter().find(|a| a.name == name).unwrap()
    }

    fn simple_expense(
        workspace: &Workspace,
        chequing: &Account,
        food: &Account,
        amount: i32,
        date: &str,
        conn: &mut SqliteConnection,
    ) -> Transaction {
        NewTransaction::new(
            workspace.as_ref().clone(),
            date,
            "Groceries",
            vec![
                PostingInput::new(chequing.id.clone(), -amount),
                PostingInput::new(food.id.clone(), amount),
            ],
        )
        .cleared(true)
        .insert(conn)
        .unwrap()
    }

    #[test]
    fn two_posting_transaction_succeeds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let transaction = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(transaction.date, "2026-01-15");
        assert_eq!(transaction.description, "Lunch");
        assert!(!transaction.cleared);
    }

    #[test]
    fn three_posting_transaction_succeeds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let dining = find_account("Dining", &workspace, &mut conn);

        let transaction = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Split meal",
            vec![
                PostingInput::new(chequing.id.clone(), -2000),
                PostingInput::new(food.id.clone(), 1200),
                PostingInput::new(dining.id.clone(), 800),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let postings = transaction.postings(&mut conn).unwrap();
        assert_eq!(postings.len(), 3);
    }

    #[test]
    fn non_zero_sum_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let err = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Bad",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1000),
            ],
        )
        .insert(&mut conn)
        .expect_err("non-zero sum should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn fewer_than_two_postings_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        let err = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Bad",
            vec![PostingInput::new(chequing.id.clone(), 0)],
        )
        .insert(&mut conn)
        .expect_err("single posting should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn zero_amount_posting_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let err = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Bad",
            vec![
                PostingInput::new(chequing.id.clone(), 0),
                PostingInput::new(food.id.clone(), 0),
            ],
        )
        .insert(&mut conn)
        .expect_err("zero amount should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn postings_returns_correct_postings() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let transaction = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let postings = transaction.postings(&mut conn).unwrap();
        assert_eq!(postings.len(), 2);

        let sum: i64 = postings.iter().map(|p| p.amount as i64).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn for_account_returns_matching_transactions() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        // Transaction touching chequing + food
        simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        // Transaction touching chequing + rent
        NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-16",
            "Rent payment",
            vec![
                PostingInput::new(chequing.id.clone(), -150000),
                PostingInput::new(rent.id.clone(), 150000),
            ],
        )
        .cleared(true)
        .insert(&mut conn)
        .unwrap();

        // Food account should see only 1 transaction
        let food_transactions = Transaction::for_account(&food.id, &mut conn).unwrap();
        assert_eq!(food_transactions.len(), 1);
        assert_eq!(food_transactions[0].description, "Groceries");

        // Chequing should see both
        let chequing_transactions = Transaction::for_account(&chequing.id, &mut conn).unwrap();
        assert_eq!(chequing_transactions.len(), 2);
    }

    #[test]
    fn soft_deleted_transactions_excluded_from_queries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let transaction = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        transaction.soft_delete(&mut conn).unwrap();

        let transactions = Transaction::for_account(&food.id, &mut conn).unwrap();
        assert!(transactions.is_empty());
    }

    #[test]
    fn cleared_flag_set_via_builder() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let transaction = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200),
            ],
        )
        .cleared(true)
        .insert(&mut conn)
        .unwrap();

        assert!(transaction.cleared);
    }

    #[test]
    fn posting_note_stored() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let transaction = NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-15",
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200).note("tacos"),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let postings = transaction.postings(&mut conn).unwrap();
        let food_posting = postings.iter().find(|p| p.amount == 1200).unwrap();
        assert_eq!(food_posting.note, "tacos");
    }

    #[test]
    fn for_workspace_returns_all_workspace_transactions() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-16",
            "Rent",
            vec![
                PostingInput::new(chequing.id.clone(), -150000),
                PostingInput::new(rent.id.clone(), 150000),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let all = Transaction::for_workspace(workspace.as_ref(), &mut conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn for_account_tree_includes_descendants() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let expenses = find_account("Expenses", &workspace, &mut conn);

        // Create sub-account under Food
        let cafes = NewAccount::child(&food, "Cafes").insert(&mut conn).unwrap();

        // Post to Food directly
        simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        // Post to Cafes (child of Food)
        NewTransaction::new(
            workspace.as_ref().clone(),
            "2026-01-16",
            "Coffee",
            vec![
                PostingInput::new(chequing.id.clone(), -500),
                PostingInput::new(cafes.id.clone(), 500),
            ],
        )
        .cleared(true)
        .insert(&mut conn)
        .unwrap();

        // for_account_tree on Expenses should find both (Food and Cafes are descendants)
        let transactions = Transaction::for_account_tree(&expenses, &mut conn).unwrap();
        assert_eq!(transactions.len(), 2);

        // for_account on Expenses directly should find none (no posting directly to Expenses)
        let direct = Transaction::for_account(&expenses.id, &mut conn).unwrap();
        assert!(direct.is_empty());
    }
}
