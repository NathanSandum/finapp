use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::AppError;
use crate::schema::users;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser {
    id: String,
    email: String,
    display_name: String,
    password_hash: String,
    created_at: String,
    updated_at: String,
}

pub struct UserRepository;

impl UserRepository {
    pub fn create(
        conn: &mut SqliteConnection,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::Internal)?
            .to_string();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let new_user = NewUser {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            password_hash,
            created_at: now.clone(),
            updated_at: now,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
            .map_err(|e| match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => AppError::Conflict(format!("email '{}' is already in use", email)),
                _ => AppError::Internal,
            })
    }

    pub fn find_by_email(
        conn: &mut SqliteConnection,
        email: &str,
    ) -> Result<Option<User>, AppError> {
        users::table
            .filter(users::email.eq(email))
            .select(User::as_select())
            .first(conn)
            .optional()
            .map_err(|_| AppError::Internal)
    }

    pub fn verify_password(user: &User, password: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(&user.password_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_db;

    #[test]
    fn create_user_succeeds() {
        let mut conn = setup_test_db();
        let user = UserRepository::create(&mut conn, "alice@example.com", "Alice", "password123")
            .expect("user creation should succeed");

        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.display_name, "Alice");
        assert!(!user.id.is_empty());
    }

    #[test]
    fn duplicate_email_fails() {
        let mut conn = setup_test_db();
        UserRepository::create(&mut conn, "alice@example.com", "Alice", "password123")
            .expect("first insert should succeed");

        let err = UserRepository::create(&mut conn, "alice@example.com", "Alice2", "password456")
            .expect_err("duplicate email should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn password_not_stored_in_plaintext() {
        let mut conn = setup_test_db();
        let user = UserRepository::create(&mut conn, "alice@example.com", "Alice", "secret123")
            .expect("user creation should succeed");

        assert_ne!(user.password_hash, "secret123");
        assert!(user.password_hash.starts_with("$argon2"), "expected argon2 hash");
    }

    #[test]
    fn find_by_email_returns_user() {
        let mut conn = setup_test_db();
        UserRepository::create(&mut conn, "alice@example.com", "Alice", "password123")
            .expect("user creation should succeed");

        let found = UserRepository::find_by_email(&mut conn, "alice@example.com")
            .expect("query should succeed")
            .expect("user should be found");

        assert_eq!(found.email, "alice@example.com");
    }

    #[test]
    fn find_by_email_returns_none_for_unknown() {
        let mut conn = setup_test_db();
        let found = UserRepository::find_by_email(&mut conn, "nobody@example.com")
            .expect("query should succeed");
        assert!(found.is_none());
    }

    #[test]
    fn verify_password_correct() {
        let mut conn = setup_test_db();
        let user = UserRepository::create(&mut conn, "alice@example.com", "Alice", "correct")
            .expect("user creation should succeed");
        assert!(UserRepository::verify_password(&user, "correct"));
    }

    #[test]
    fn verify_password_wrong() {
        let mut conn = setup_test_db();
        let user = UserRepository::create(&mut conn, "alice@example.com", "Alice", "correct")
            .expect("user creation should succeed");
        assert!(!UserRepository::verify_password(&user, "wrong"));
    }
}
