use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::account::AccountRef;
use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::{entries, postings};

// ── EntryRef ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct EntryRef(String);

impl std::fmt::Display for EntryRef {
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

// ── Entry ───────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, Queryable, Selectable)]
#[diesel(table_name = entries)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Entry {
    pub id: EntryRef,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    pub date: NaiveDate,
    pub description: String,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Entry {
    pub fn as_ref(&self) -> &EntryRef {
        &self.id
    }

    pub fn from_ref(
        entry_ref: &EntryRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Entry>, AppError> {
        entries::table
            .find(entry_ref)
            .select(Entry::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("entry lookup failed: {err}")))
    }

    pub fn postings(&self, conn: &mut SqliteConnection) -> Result<Vec<Posting>, AppError> {
        postings::table
            .filter(postings::entry_id.eq(&self.id))
            .select(Posting::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("postings query failed: {err}")))
    }

    pub fn update(
        &self,
        changes: UpdateEntry,
        conn: &mut SqliteConnection,
    ) -> Result<Entry, AppError> {
        if let Some(ref posting_inputs) = changes.postings {
            validate_postings(posting_inputs)?;
        }

        conn.transaction::<_, AppError, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();

            if let Some(ref date) = changes.date {
                diesel::update(entries::table.find(&self.id))
                    .set(entries::date.eq(date))
                    .execute(conn)?;
            }
            if let Some(ref description) = changes.description {
                diesel::update(entries::table.find(&self.id))
                    .set(entries::description.eq(description))
                    .execute(conn)?;
            }

            if let Some(posting_inputs) = changes.postings {
                diesel::delete(postings::table.filter(postings::entry_id.eq(&self.id)))
                    .execute(conn)?;

                for input in posting_inputs {
                    diesel::insert_into(postings::table)
                        .values(NewPostingRow {
                            id: Uuid::new_v4().to_string(),
                            entry: self.id.clone(),
                            account: input.account,
                            amount: input.amount,
                            note: input.note.unwrap_or_default(),
                            created_at: now,
                            updated_at: now,
                        })
                        .execute(conn)?;
                }
            }

            diesel::update(entries::table.find(&self.id))
                .set(entries::updated_at.eq(now))
                .execute(conn)?;

            entries::table
                .find(&self.id)
                .select(Entry::as_select())
                .first(conn)
                .map_err(|err| AppError::Internal(format!("entry refresh failed: {err}")))
        })
    }

    pub fn merge(
        source_entries: &[Entry],
        workspace: &WorkspaceRef,
        date: NaiveDate,
        description: &str,
        posting_inputs: Vec<PostingInput>,
        conn: &mut SqliteConnection,
    ) -> Result<Entry, AppError> {
        if source_entries.is_empty() {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "source_entry_ids".into(),
                message: "must provide at least one source entry".into(),
            }]));
        }

        for entry in source_entries {
            if entry.workspace != *workspace {
                return Err(AppError::Validation(vec![crate::error::FieldError {
                    field: "source_entry_ids".into(),
                    message: "all source entries must belong to the same workspace".into(),
                }]));
            }
        }

        validate_postings(&posting_inputs)?;

        conn.transaction::<_, AppError, _>(|conn| {
            for entry in source_entries {
                entry.soft_delete(conn)?;
            }

            let new_entry = NewEntry::new(
                workspace.clone(),
                date,
                description,
                posting_inputs,
            )
            .insert(conn)?;

            Ok(new_entry)
        })
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(entries::table.find(&self.id))
            .set(entries::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("entry soft delete failed: {err}")))?;
        Ok(())
    }
}

// ── Query methods on owning types ────────────────────────────────────────────

impl AccountRef {
    /// Entries with any posting to this account, excluding soft-deleted, ordered by date desc.
    pub fn entries(&self, conn: &mut SqliteConnection) -> Result<Vec<Entry>, AppError> {
        entries::table
            .inner_join(postings::table.on(postings::entry_id.eq(entries::id)))
            .filter(postings::account_id.eq(self))
            .filter(entries::deleted_at.is_null())
            .select(Entry::as_select())
            .distinct()
            .order(entries::date.desc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("account entries query failed: {err}")))
    }
}

impl crate::domain::account::Account {
    /// Entries with postings to this account or any of its descendants.
    /// Use case: viewing all activity under a parent account (e.g. all Expense entries
    /// when looking at the Expenses root, including Food, Rent, etc.).
    pub fn entries_with_descendants(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Entry>, AppError> {
        let descendants = self.descendants(conn)?;
        let mut account_ids: Vec<&AccountRef> = descendants.iter().map(|a| &a.id).collect();
        account_ids.push(&self.id);

        entries::table
            .inner_join(postings::table.on(postings::entry_id.eq(entries::id)))
            .filter(postings::account_id.eq_any(&account_ids))
            .filter(entries::deleted_at.is_null())
            .select(Entry::as_select())
            .distinct()
            .order(entries::date.desc())
            .load(conn)
            .map_err(|err| {
                AppError::Internal(format!("account tree entries query failed: {err}"))
            })
    }
}

impl WorkspaceRef {
    pub fn entries(&self, conn: &mut SqliteConnection) -> Result<Vec<Entry>, AppError> {
        entries::table
            .filter(entries::workspace_id.eq(self))
            .filter(entries::deleted_at.is_null())
            .select(Entry::as_select())
            .order(entries::date.desc())
            .load(conn)
            .map_err(|err| {
                AppError::Internal(format!("workspace entries query failed: {err}"))
            })
    }

    pub fn entries_between(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Entry>, AppError> {
        entries::table
            .filter(entries::workspace_id.eq(self))
            .filter(entries::deleted_at.is_null())
            .filter(entries::date.ge(start_date))
            .filter(entries::date.le(end_date))
            .select(Entry::as_select())
            .order(entries::date.desc())
            .load(conn)
            .map_err(|err| {
                AppError::Internal(format!("workspace entries query failed: {err}"))
            })
    }
}

// ── Posting ─────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, Queryable, Selectable)]
#[diesel(table_name = postings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Posting {
    pub id: PostingRef,
    #[diesel(column_name = entry_id)]
    pub entry: EntryRef,
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

// ── NewEntry ────────────────────────────────────────────────────────────────

pub struct PostingInput {
    pub account: AccountRef,
    pub amount: i32,
    pub note: Option<String>,
}

fn validate_postings(posting_inputs: &[PostingInput]) -> Result<(), AppError> {
    if posting_inputs.len() < 2 {
        return Err(AppError::Validation(vec![crate::error::FieldError {
            field: "postings".into(),
            message: "entry must have at least 2 postings".into(),
        }]));
    }

    for input in posting_inputs {
        if input.amount == 0 {
            return Err(AppError::Validation(vec![crate::error::FieldError {
                field: "postings".into(),
                message: "posting amount cannot be zero".into(),
            }]));
        }
    }

    let sum: i64 = posting_inputs.iter().map(|p| p.amount as i64).sum();
    if sum != 0 {
        return Err(AppError::Validation(vec![crate::error::FieldError {
            field: "postings".into(),
            message: format!("postings must sum to zero, got {sum}"),
        }]));
    }

    Ok(())
}

pub struct UpdateEntry {
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub postings: Option<Vec<PostingInput>>,
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
#[diesel(table_name = entries)]
struct NewEntryRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    date: NaiveDate,
    description: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = postings)]
struct NewPostingRow {
    id: String,
    #[diesel(column_name = entry_id)]
    entry: EntryRef,
    #[diesel(column_name = account_id)]
    account: AccountRef,
    amount: i32,
    note: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewEntry {
    workspace: WorkspaceRef,
    date: NaiveDate,
    description: String,
    posting_inputs: Vec<PostingInput>,
}

impl NewEntry {
    pub fn new(
        workspace: WorkspaceRef,
        date: NaiveDate,
        description: &str,
        posting_inputs: Vec<PostingInput>,
    ) -> Self {
        NewEntry {
            workspace,
            date,
            description: description.to_string(),
            posting_inputs,
        }
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Entry, AppError> {
        validate_postings(&self.posting_inputs)?;

        conn.transaction::<_, AppError, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();

            let entry = diesel::insert_into(entries::table)
                .values(NewEntryRow {
                    id: Uuid::new_v4().to_string(),
                    workspace: self.workspace,
                    date: self.date,
                    description: self.description,
                    created_at: now,
                    updated_at: now,
                })
                .returning(Entry::as_returning())
                .get_result(conn)?;

            for input in self.posting_inputs {
                diesel::insert_into(postings::table)
                    .values(NewPostingRow {
                        id: Uuid::new_v4().to_string(),
                        entry: entry.id.clone(),
                        account: input.account,
                        amount: input.amount,
                        note: input.note.unwrap_or_default(),
                        created_at: now,
                        updated_at: now,
                    })
                    .execute(conn)?;
            }

            Ok(entry)
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

    fn nd(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn simple_expense(
        workspace: &Workspace,
        chequing: &Account,
        food: &Account,
        amount: i32,
        date: &str,
        conn: &mut SqliteConnection,
    ) -> Entry {
        NewEntry::new(
            workspace.as_ref().clone(),
            nd(date),
            "Groceries",
            vec![
                PostingInput::new(chequing.id.clone(), -amount),
                PostingInput::new(food.id.clone(), amount),
            ],
        )
        .insert(conn)
        .unwrap()
    }

    #[test]
    fn two_posting_entry_succeeds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(entry.date, nd("2026-01-15"));
        assert_eq!(entry.description, "Lunch");
    }

    #[test]
    fn three_posting_entry_succeeds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let dining = find_account("Dining", &workspace, &mut conn);

        let entry = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
            "Split meal",
            vec![
                PostingInput::new(chequing.id.clone(), -2000),
                PostingInput::new(food.id.clone(), 1200),
                PostingInput::new(dining.id.clone(), 800),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let postings = entry.postings(&mut conn).unwrap();
        assert_eq!(postings.len(), 3);
    }

    #[test]
    fn non_zero_sum_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let err = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
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

        let err = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
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

        let err = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
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

        let entry = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let postings = entry.postings(&mut conn).unwrap();
        assert_eq!(postings.len(), 2);

        let sum: i64 = postings.iter().map(|p| p.amount as i64).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn for_account_returns_matching_entries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        // Entry touching chequing + food
        simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        // Entry touching chequing + rent
        NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-16"),
            "Rent payment",
            vec![
                PostingInput::new(chequing.id.clone(), -150000),
                PostingInput::new(rent.id.clone(), 150000),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        // Food account should see only 1 entry
        let food_entries = food.id.entries(&mut conn).unwrap();
        assert_eq!(food_entries.len(), 1);
        assert_eq!(food_entries[0].description, "Groceries");

        // Chequing should see both
        let chequing_entries = chequing.id.entries(&mut conn).unwrap();
        assert_eq!(chequing_entries.len(), 2);
    }

    #[test]
    fn soft_deleted_entries_excluded_from_queries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        entry.soft_delete(&mut conn).unwrap();

        let entries = food.id.entries(&mut conn).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn posting_note_stored() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-15"),
            "Lunch",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200).note("tacos"),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let postings = entry.postings(&mut conn).unwrap();
        let food_posting = postings.iter().find(|p| p.amount == 1200).unwrap();
        assert_eq!(food_posting.note, "tacos");
    }

    #[test]
    fn for_workspace_returns_all_workspace_entries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-16"),
            "Rent",
            vec![
                PostingInput::new(chequing.id.clone(), -150000),
                PostingInput::new(rent.id.clone(), 150000),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let all = workspace.as_ref().entries(&mut conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn update_date_only() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        let original_postings = entry.postings(&mut conn).unwrap();

        let updated = entry
            .update(
                UpdateEntry {
                    date: Some(nd("2026-02-20")),
                    description: None,
                    postings: None,
                },
                &mut conn,
            )
            .unwrap();

        assert_eq!(updated.date, nd("2026-02-20"));
        assert_eq!(updated.description, "Groceries");

        let postings_after = updated.postings(&mut conn).unwrap();
        assert_eq!(postings_after.len(), original_postings.len());
        for (before, after) in original_postings.iter().zip(postings_after.iter()) {
            assert_eq!(before.id, after.id);
        }
    }

    #[test]
    fn update_description_only() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let updated = entry
            .update(
                UpdateEntry {
                    date: None,
                    description: Some("Updated description".to_string()),
                    postings: None,
                },
                &mut conn,
            )
            .unwrap();

        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.date, nd("2026-01-15"));
    }

    #[test]
    fn update_replaces_postings() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        let old_postings = entry.postings(&mut conn).unwrap();

        let updated = entry
            .update(
                UpdateEntry {
                    date: None,
                    description: None,
                    postings: Some(vec![
                        PostingInput::new(chequing.id.clone(), -5000),
                        PostingInput::new(rent.id.clone(), 5000),
                    ]),
                },
                &mut conn,
            )
            .unwrap();

        let new_postings = updated.postings(&mut conn).unwrap();
        assert_eq!(new_postings.len(), 2);

        // Old posting IDs should be gone
        for old in &old_postings {
            assert!(
                !new_postings.iter().any(|p| p.id == old.id),
                "old posting should have been deleted"
            );
        }

        // New postings should reference rent, not food
        let accounts: Vec<&AccountRef> = new_postings.iter().map(|p| &p.account).collect();
        assert!(accounts.contains(&&rent.id));
        assert!(!accounts.contains(&&food.id));
    }

    #[test]
    fn update_postings_non_zero_sum_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let error = entry
            .update(
                UpdateEntry {
                    date: None,
                    description: None,
                    postings: Some(vec![
                        PostingInput::new(chequing.id.clone(), -1200),
                        PostingInput::new(food.id.clone(), 1000),
                    ]),
                },
                &mut conn,
            )
            .expect_err("non-zero sum should fail");

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn update_postings_fewer_than_two_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let error = entry
            .update(
                UpdateEntry {
                    date: None,
                    description: None,
                    postings: Some(vec![PostingInput::new(chequing.id.clone(), 100)]),
                },
                &mut conn,
            )
            .expect_err("single posting should fail");

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn update_postings_zero_amount_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let error = entry
            .update(
                UpdateEntry {
                    date: None,
                    description: None,
                    postings: Some(vec![
                        PostingInput::new(chequing.id.clone(), 0),
                        PostingInput::new(food.id.clone(), 0),
                    ]),
                },
                &mut conn,
            )
            .expect_err("zero amount should fail");

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn update_no_changes() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let updated = entry
            .update(
                UpdateEntry {
                    date: None,
                    description: None,
                    postings: None,
                },
                &mut conn,
            )
            .unwrap();

        assert_eq!(updated.date, entry.date);
        assert_eq!(updated.description, entry.description);
        assert!(updated.updated_at >= entry.updated_at);
    }

    #[test]
    fn merge_two_entries_into_one() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let savings = find_account("Savings", &workspace, &mut conn);

        let entry_one = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);
        let entry_two = NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-16"),
            "ATM withdrawal",
            vec![
                PostingInput::new(chequing.id.clone(), -5000),
                PostingInput::new(savings.id.clone(), 5000),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        let merged = Entry::merge(
            &[entry_one, entry_two],
            workspace.as_ref(),
            nd("2026-01-15"),
            "Transfer to savings",
            vec![
                PostingInput::new(chequing.id.clone(), -6200),
                PostingInput::new(savings.id.clone(), 6200),
            ],
            &mut conn,
        )
        .unwrap();

        assert_eq!(merged.description, "Transfer to savings");
        assert_eq!(merged.date, nd("2026-01-15"));

        let postings = merged.postings(&mut conn).unwrap();
        assert_eq!(postings.len(), 2);
        let sum: i64 = postings.iter().map(|p| p.amount as i64).sum();
        assert_eq!(sum, 0);

        // Originals should be soft-deleted (excluded from workspace entries)
        let all_entries = workspace.as_ref().entries(&mut conn).unwrap();
        assert_eq!(all_entries.len(), 1);
        assert_eq!(all_entries[0].id, merged.id);
    }

    #[test]
    fn merge_rejects_non_zero_sum_postings() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        let error = Entry::merge(
            &[entry],
            workspace.as_ref(),
            nd("2026-01-15"),
            "Bad merge",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1000),
            ],
            &mut conn,
        )
        .expect_err("non-zero sum should fail");

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn merge_rejects_empty_sources() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let error = Entry::merge(
            &[],
            workspace.as_ref(),
            nd("2026-01-15"),
            "Empty merge",
            vec![
                PostingInput::new(chequing.id.clone(), -1200),
                PostingInput::new(food.id.clone(), 1200),
            ],
            &mut conn,
        )
        .expect_err("empty sources should fail");

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn merge_cross_workspace_rejected() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let entry = simple_expense(&workspace, &chequing, &food, 1200, "2026-01-15", &mut conn);

        // Create a second workspace
        let user_two = NewUser::new("bob@example.com", "Bob", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace_two = NewWorkspace::new("Other Finances", user_two.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        let other_chequing = find_account("Chequing", &workspace_two, &mut conn);
        let other_food = find_account("Food", &workspace_two, &mut conn);

        let other_entry = simple_expense(
            &workspace_two,
            &other_chequing,
            &other_food,
            500,
            "2026-01-15",
            &mut conn,
        );

        let error = Entry::merge(
            &[entry, other_entry],
            workspace.as_ref(),
            nd("2026-01-15"),
            "Cross workspace",
            vec![
                PostingInput::new(chequing.id.clone(), -1700),
                PostingInput::new(food.id.clone(), 1700),
            ],
            &mut conn,
        )
        .expect_err("cross-workspace should fail");

        assert!(matches!(error, AppError::Validation(_)));
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
        NewEntry::new(
            workspace.as_ref().clone(),
            nd("2026-01-16"),
            "Coffee",
            vec![
                PostingInput::new(chequing.id.clone(), -500),
                PostingInput::new(cafes.id.clone(), 500),
            ],
        )
        .insert(&mut conn)
        .unwrap();

        // entries_with_descendants on Expenses should find both (Food and Cafes are descendants)
        let entries = expenses.entries_with_descendants(&mut conn).unwrap();
        assert_eq!(entries.len(), 2);

        // entries on Expenses directly should find none (no posting directly to Expenses)
        let direct = expenses.id.entries(&mut conn).unwrap();
        assert!(direct.is_empty());
    }
}
