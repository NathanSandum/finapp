use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use uuid::Uuid;

use crate::domain::workspace::Workspace;
use crate::error::AppError;
use crate::schema::categories;

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct CategoryRef {
    pub id: String,
}

impl CategoryRef {
    pub fn new(id: &str) -> Self {
        CategoryRef { id: id.to_string() }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Category {
    pub fn as_ref(&self) -> CategoryRef {
        CategoryRef { id: self.id.clone() }
    }

    pub fn from_ref(category_ref: &CategoryRef, conn: &mut SqliteConnection) -> Result<Option<Category>, AppError> {
        categories::table
            .find(&category_ref.id)
            .select(Category::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("category lookup failed: {err}")))
    }

    pub fn from_id(id: &str, conn: &mut SqliteConnection) -> Result<Option<Category>, AppError> {
        Self::from_ref(&CategoryRef::new(id), conn)
    }

    pub fn for_workspace(
        workspace: &Workspace,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Category>, AppError> {
        categories::table
            .filter(categories::workspace_id.eq(&workspace.id))
            .filter(categories::deleted_at.is_null())
            .select(Category::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("category list failed: {err}")))
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(categories::table.find(&self.id))
            .set(categories::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("category soft delete failed: {err}")))?;
        Ok(())
    }
}

// ── NewCategory ──────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = categories)]
struct NewCategoryRow {
    id: String,
    workspace_id: String,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewCategory {
    workspace_id: String,
    name: String,
    description: Option<String>,
}

impl NewCategory {
    pub fn new(workspace: &Workspace, name: &str) -> Self {
        NewCategory {
            workspace_id: workspace.id.clone(),
            name: name.to_string(),
            description: None,
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Category, AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::insert_into(categories::table)
            .values(NewCategoryRow {
                id: Uuid::new_v4().to_string(),
                workspace_id: self.workspace_id,
                name: self.name.clone(),
                description: self.description,
                created_at: now,
                updated_at: now,
            })
            .returning(Category::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!("category '{}' already exists in this workspace", self.name))
                }
                other => AppError::Internal(format!("category insert failed: {other}")),
            })
    }
}

/// Default categories created with every new workspace.
pub const DEFAULT_CATEGORIES: &[&str] = &[
    "Employment Income",
    "Rent",
    "Groceries",
    "Dining",
    "Transit",
    "Utilities",
    "Entertainment",
    "Other",
];

pub fn create_defaults(
    workspace: &Workspace,
    conn: &mut SqliteConnection,
) -> Result<Vec<Category>, AppError> {
    DEFAULT_CATEGORIES
        .iter()
        .map(|name| NewCategory::new(workspace, name).insert(conn))
        .collect()
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
            .expect("new user should succeed")
            .insert(&mut conn)
            .expect("user insert should succeed");
        let ws = NewWorkspace::new("My Finances", &user)
            .insert(&mut conn)
            .expect("workspace creation should succeed");
        (conn, ws)
    }

    #[test]
    fn create_category_succeeds() {
        let (mut conn, ws) = setup();
        let cat = NewCategory::new(&ws, "Groceries")
            .description("Food and household items")
            .insert(&mut conn)
            .expect("category creation should succeed");

        assert_eq!(cat.name, "Groceries");
        assert_eq!(cat.description, Some("Food and household items".to_string()));
        assert_eq!(cat.workspace_id, ws.id);
    }

    #[test]
    fn duplicate_name_in_same_workspace_fails() {
        let (mut conn, ws) = setup();
        NewCategory::new(&ws, "Groceries")
            .insert(&mut conn)
            .expect("first insert should succeed");

        let err = NewCategory::new(&ws, "Groceries")
            .insert(&mut conn)
            .expect_err("duplicate should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_name_in_different_workspaces_succeeds() {
        let mut conn = setup_test_db();
        let user_a = NewUser::new("a@example.com", "A", "password")
            .expect("new user should succeed")
            .insert(&mut conn)
            .expect("insert should succeed");
        let user_b = NewUser::new("b@example.com", "B", "password")
            .expect("new user should succeed")
            .insert(&mut conn)
            .expect("insert should succeed");
        let ws_a = NewWorkspace::new("WS A", &user_a).insert(&mut conn).unwrap();
        let ws_b = NewWorkspace::new("WS B", &user_b).insert(&mut conn).unwrap();

        NewCategory::new(&ws_a, "Groceries").insert(&mut conn).unwrap();
        NewCategory::new(&ws_b, "Groceries").insert(&mut conn).unwrap();
    }

    #[test]
    fn for_workspace_returns_only_active_categories() {
        let (mut conn, ws) = setup();
        NewCategory::new(&ws, "Groceries").insert(&mut conn).unwrap();
        let dining = NewCategory::new(&ws, "Dining").insert(&mut conn).unwrap();
        dining.soft_delete(&mut conn).unwrap();

        let cats = Category::for_workspace(&ws, &mut conn).unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "Groceries");
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

        NewCategory::new(&ws_a, "A's Category").insert(&mut conn).unwrap();
        NewCategory::new(&ws_b, "B's Category").insert(&mut conn).unwrap();

        let cats = Category::for_workspace(&ws_a, &mut conn).unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "A's Category");
    }

    #[test]
    fn create_defaults_creates_all_default_categories() {
        let (mut conn, ws) = setup();
        let cats = create_defaults(&ws, &mut conn).unwrap();

        assert_eq!(cats.len(), DEFAULT_CATEGORIES.len());
        let names: Vec<&str> = cats.iter().map(|c| c.name.as_str()).collect();
        for &default in DEFAULT_CATEGORIES {
            assert!(names.contains(&default), "missing default category: {default}");
        }
    }

    #[test]
    fn soft_deleted_category_is_recoverable() {
        let (mut conn, ws) = setup();
        let cat = NewCategory::new(&ws, "Groceries").insert(&mut conn).unwrap();
        cat.soft_delete(&mut conn).unwrap();

        // Still exists in DB, just hidden from for_workspace
        let all: Vec<Category> = categories::table
            .filter(categories::workspace_id.eq(&ws.id))
            .select(Category::as_select())
            .load(&mut conn)
            .unwrap();
        assert_eq!(all.len(), 1);
        assert!(all[0].deleted_at.is_some());
    }
}
