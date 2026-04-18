use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use uuid::Uuid;

use crate::error::AppError;
use crate::schema::users;

/// Lightweight identity reference — just an id, no DB load required.
/// Deserializes from a plain string (e.g. in path parameters).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct UserRef(String);

impl UserRef {
    pub fn new(id: &str) -> Self {
        UserRef(id.to_string())
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn as_ref(&self) -> UserRef {
        UserRef(self.id.clone())
    }

    pub fn from_ref(user_ref: &UserRef, conn: &mut SqliteConnection) -> Result<Option<User>, AppError> {
        users::table
            .find(&user_ref.0)
            .select(User::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("user lookup failed: {err}")))
    }

    pub fn from_email(email: &str, conn: &mut SqliteConnection) -> Result<Option<User>, AppError> {
        users::table
            .filter(users::email.eq(email))
            .select(User::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("user lookup failed: {err}")))
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(&self.password_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    id: String,
    email: String,
    display_name: String,
    password_hash: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl NewUser {
    pub fn new(email: &str, display_name: &str, password: &str) -> Result<Self, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|err| AppError::Internal(format!("password hash failed: {err}")))?
            .to_string();

        let now = chrono::Utc::now().naive_utc();
        Ok(NewUser {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            password_hash,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<User, AppError> {
        let email = self.email.clone();
        diesel::insert_into(users::table)
            .values(&self)
            .returning(User::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!("email '{email}' is already in use"))
                }
                other => AppError::Internal(format!("user insert failed: {other}")),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_db;

    fn create_user(conn: &mut SqliteConnection, email: &str) -> User {
        NewUser::new(email, "Test User", "password123")
            .expect("new user should succeed")
            .insert(conn)
            .expect("insert should succeed")
    }

    #[test]
    fn create_user_succeeds() {
        let mut conn = setup_test_db();
        let user = create_user(&mut conn, "alice@example.com");

        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.display_name, "Test User");
        assert!(!user.id.is_empty());
    }

    #[test]
    fn duplicate_email_fails() {
        let mut conn = setup_test_db();
        create_user(&mut conn, "alice@example.com");

        let err = NewUser::new("alice@example.com", "Alice2", "password456")
            .expect("new user should succeed")
            .insert(&mut conn)
            .expect_err("duplicate email should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn password_not_stored_in_plaintext() {
        let mut conn = setup_test_db();
        let user = create_user(&mut conn, "alice@example.com");

        assert_ne!(user.password_hash, "password123");
        assert!(user.password_hash.starts_with("$argon2"), "expected argon2 hash");
    }

    #[test]
    fn from_email_returns_user() {
        let mut conn = setup_test_db();
        create_user(&mut conn, "alice@example.com");

        let found = User::from_email("alice@example.com", &mut conn)
            .expect("query should succeed")
            .expect("user should be found");

        assert_eq!(found.email, "alice@example.com");
    }

    #[test]
    fn from_email_returns_none_for_unknown() {
        let mut conn = setup_test_db();
        let found = User::from_email("nobody@example.com", &mut conn)
            .expect("query should succeed");
        assert!(found.is_none());
    }

    #[test]
    fn verify_password_correct() {
        let mut conn = setup_test_db();
        let user = create_user(&mut conn, "alice@example.com");
        assert!(user.verify_password("password123"));
    }

    #[test]
    fn verify_password_wrong() {
        let mut conn = setup_test_db();
        let user = create_user(&mut conn, "alice@example.com");
        assert!(!user.verify_password("wrong"));
    }
}
