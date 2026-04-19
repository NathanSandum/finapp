use chrono::NaiveDateTime;
use diesel::prelude::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::user::User;
use crate::error::AppError;
use crate::schema::sessions;

const SESSION_DURATION_HOURS: i64 = 24 * 7; // 1 week

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: String,
    #[diesel(column_name = user_id)]
    pub user: String,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl Session {
    /// Validate a raw token and return the associated user if valid and not expired.
    pub fn validate(
        token: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Option<User>, AppError> {
        use crate::schema::users;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now().naive_utc();

        sessions::table
            .inner_join(users::table)
            .filter(sessions::token_hash.eq(&token_hash))
            .filter(sessions::expires_at.gt(&now))
            .select(User::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("session validation failed: {err}")))
    }

    /// Invalidate a session by raw token (logout).
    pub fn invalidate(token: &str, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let token_hash = hash_token(token);
        diesel::delete(sessions::table.filter(sessions::token_hash.eq(&token_hash)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("session invalidation failed: {err}")))?;
        Ok(())
    }
}

// ── NewSession ───────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = sessions)]
struct NewSessionRow {
    id: String,
    #[diesel(column_name = user_id)]
    user: String,
    token_hash: String,
    expires_at: NaiveDateTime,
    created_at: NaiveDateTime,
}

/// Result of creating a session — includes the raw token (only available at creation time).
pub struct SessionToken {
    pub token: String,
    pub expires_at: NaiveDateTime,
}

pub struct NewSession;

impl NewSession {
    /// Create a session for a user. Returns the raw token (never stored).
    pub fn create(user: &User, conn: &mut SqliteConnection) -> Result<SessionToken, AppError> {
        let token = Uuid::new_v4().to_string();
        let token_hash = hash_token(&token);
        let now = chrono::Utc::now().naive_utc();
        let expires_at = now + chrono::Duration::hours(SESSION_DURATION_HOURS);

        diesel::insert_into(sessions::table)
            .values(NewSessionRow {
                id: Uuid::new_v4().to_string(),
                user: user.id.clone(),
                token_hash,
                expires_at,
                created_at: now,
            })
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("session creation failed: {err}")))?;

        Ok(SessionToken { token, expires_at })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::NewUser;
    use crate::test_helpers::setup_test_db;

    fn make_user(conn: &mut SqliteConnection) -> User {
        NewUser::new("alice@example.com", "Alice", "password")
            .unwrap()
            .insert(conn)
            .unwrap()
    }

    #[test]
    fn create_session_returns_token() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn);

        let session = NewSession::create(&user, &mut conn)
            .expect("session creation should succeed");

        assert!(!session.token.is_empty());
        assert!(session.expires_at > chrono::Utc::now().naive_utc());
    }

    #[test]
    fn validate_valid_token_returns_user() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn);
        let session = NewSession::create(&user, &mut conn).unwrap();

        let found = Session::validate(&session.token, &mut conn)
            .expect("validation should succeed")
            .expect("user should be found");

        assert_eq!(found.id, user.id);
        assert_eq!(found.email, user.email);
    }

    #[test]
    fn validate_invalid_token_returns_none() {
        let mut conn = setup_test_db();
        let _user = make_user(&mut conn);

        let found = Session::validate("bogus-token", &mut conn)
            .expect("validation should succeed");

        assert!(found.is_none());
    }

    #[test]
    fn validate_expired_token_returns_none() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn);

        // Insert a session that's already expired
        let token = Uuid::new_v4().to_string();
        let token_hash = hash_token(&token);
        let now = chrono::Utc::now().naive_utc();
        let expired = now - chrono::Duration::hours(1);

        diesel::insert_into(sessions::table)
            .values(NewSessionRow {
                id: Uuid::new_v4().to_string(),
                user: user.id.clone(),
                token_hash,
                expires_at: expired,
                created_at: now,
            })
            .execute(&mut conn)
            .unwrap();

        let found = Session::validate(&token, &mut conn)
            .expect("validation should succeed");

        assert!(found.is_none(), "expired token should not validate");
    }

    #[test]
    fn invalidate_removes_session() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn);
        let session = NewSession::create(&user, &mut conn).unwrap();

        Session::invalidate(&session.token, &mut conn)
            .expect("invalidation should succeed");

        let found = Session::validate(&session.token, &mut conn)
            .expect("validation should succeed");

        assert!(found.is_none(), "invalidated token should not validate");
    }

    #[test]
    fn token_is_not_stored_in_plaintext() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn);
        let session = NewSession::create(&user, &mut conn).unwrap();

        // Check the stored hash doesn't match the raw token
        let stored: Vec<String> = sessions::table
            .filter(sessions::user_id.eq(&user.id))
            .select(sessions::token_hash)
            .load(&mut conn)
            .unwrap();

        assert_eq!(stored.len(), 1);
        assert_ne!(stored[0], session.token, "raw token should not be stored");
        assert_eq!(stored[0], hash_token(&session.token));
    }
}
