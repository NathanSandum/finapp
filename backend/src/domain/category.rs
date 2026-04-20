use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::workspace::{Workspace, WorkspaceRef};
use crate::error::AppError;
use crate::schema::categories;

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct CategoryRef(String);

impl std::fmt::Display for CategoryRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Queryable, Selectable, serde::Serialize)]
#[diesel(table_name = categories)]
pub struct Category {
    #[serde(rename = "category")]
    pub id: CategoryRef,
    #[diesel(column_name = workspace_id)]
    #[serde(skip_serializing)]
    pub workspace: WorkspaceRef,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Category {
    pub fn as_ref(&self) -> CategoryRef {
        self.id.clone()
    }

    pub fn from_ref(
        category_ref: &CategoryRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Category>, AppError> {
        categories::table
            .find(category_ref)
            .select(Category::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("category lookup failed: {err}")))
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

impl WorkspaceRef {
    pub fn categories(&self, conn: &mut SqliteConnection) -> Result<Vec<Category>, AppError> {
        categories::table
            .filter(categories::workspace_id.eq(self))
            .filter(categories::deleted_at.is_null())
            .select(Category::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("category list failed: {err}")))
    }
}

// ── NewCategory ──────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = categories)]
struct NewCategoryRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewCategory {
    workspace: WorkspaceRef,
    name: String,
    description: Option<String>,
}

impl NewCategory {
    pub fn new(workspace: &WorkspaceRef, name: &str) -> Self {
        NewCategory {
            workspace: workspace.clone(),
            name: name.to_string(),
            description: None,
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Category, AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::insert_into(categories::table)
            .values(NewCategoryRow {
                id: Uuid::new_v4().to_string(),
                workspace: self.workspace,
                name: self.name.clone(),
                description: self.description,
                created_at: now,
                updated_at: now,
            })
            .returning(Category::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "category '{}' already exists in this workspace",
                        self.name
                    ))
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
    workspace: &WorkspaceRef,
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
        let workspace = NewWorkspace::new("My Finances", user.as_ref())
            .insert(&mut conn)
            .expect("workspace creation should succeed");
        (conn, workspace)
    }

    #[test]
    fn create_category_succeeds() {
        let (mut conn, workspace) = setup();
        let category = NewCategory::new(&workspace.as_ref(), "Groceries")
            .description("Food and household items")
            .insert(&mut conn)
            .expect("category creation should succeed");

        assert_eq!(category.name, "Groceries");
        assert_eq!(
            category.description,
            Some("Food and household items".to_string())
        );
        assert_eq!(category.workspace, workspace.id);
    }

    #[test]
    fn duplicate_name_in_same_workspace_fails() {
        let (mut conn, workspace) = setup();
        NewCategory::new(&workspace.as_ref(), "Groceries")
            .insert(&mut conn)
            .expect("first insert should succeed");

        let err = NewCategory::new(&workspace.as_ref(), "Groceries")
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
        let workspace_a = NewWorkspace::new("WS A", user_a.as_ref())
            .insert(&mut conn)
            .unwrap();
        let workspace_b = NewWorkspace::new("WS B", user_b.as_ref())
            .insert(&mut conn)
            .unwrap();

        NewCategory::new(&workspace_a.as_ref(), "Groceries")
            .insert(&mut conn)
            .unwrap();
        NewCategory::new(&workspace_b.as_ref(), "Groceries")
            .insert(&mut conn)
            .unwrap();
    }

    #[test]
    fn categories_returns_only_active() {
        let (mut conn, workspace) = setup();
        NewCategory::new(&workspace.as_ref(), "Groceries")
            .insert(&mut conn)
            .unwrap();
        let dining = NewCategory::new(&workspace.as_ref(), "Dining")
            .insert(&mut conn)
            .unwrap();
        dining.soft_delete(&mut conn).unwrap();

        let categories = workspace.as_ref().categories(&mut conn).unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].name, "Groceries");
    }

    #[test]
    fn categories_excludes_other_workspaces() {
        let mut conn = setup_test_db();
        let user_a = NewUser::new("a@example.com", "A", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let user_b = NewUser::new("b@example.com", "B", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace_a = NewWorkspace::new("WS A", user_a.as_ref())
            .insert(&mut conn)
            .unwrap();
        let workspace_b = NewWorkspace::new("WS B", user_b.as_ref())
            .insert(&mut conn)
            .unwrap();

        NewCategory::new(&workspace_a.as_ref(), "A's Category")
            .insert(&mut conn)
            .unwrap();
        NewCategory::new(&workspace_b.as_ref(), "B's Category")
            .insert(&mut conn)
            .unwrap();

        let categories = workspace_a.as_ref().categories(&mut conn).unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].name, "A's Category");
    }

    #[test]
    fn create_defaults_creates_all_default_categories() {
        let (mut conn, workspace) = setup();
        let categories = create_defaults(&workspace.as_ref(), &mut conn).unwrap();

        assert_eq!(categories.len(), DEFAULT_CATEGORIES.len());
        let names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
        for &default in DEFAULT_CATEGORIES {
            assert!(
                names.contains(&default),
                "missing default category: {default}"
            );
        }
    }

    #[test]
    fn soft_deleted_category_is_recoverable() {
        let (mut conn, workspace) = setup();
        let category = NewCategory::new(&workspace.as_ref(), "Groceries")
            .insert(&mut conn)
            .unwrap();
        category.soft_delete(&mut conn).unwrap();

        let all: Vec<Category> = categories::table
            .filter(categories::workspace_id.eq(&workspace.id))
            .select(Category::as_select())
            .load(&mut conn)
            .unwrap();
        assert_eq!(all.len(), 1);
        assert!(all[0].deleted_at.is_some());
    }
}
