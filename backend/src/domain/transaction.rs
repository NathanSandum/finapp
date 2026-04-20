use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;

use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::transactions;

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

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(transactions::table.find(&self.id))
            .set(transactions::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("transaction soft delete failed: {err}")))?;
        Ok(())
    }
}

// TODO: Wave 2 — postings, NewTransaction, balance computation
