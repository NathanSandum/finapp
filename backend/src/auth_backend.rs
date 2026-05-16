use serde::Deserialize;

use crate::domain::user::{User, UserRef};
use crate::error::AppError;
use crate::DbPool;

#[derive(Clone, Debug)]
pub struct AuthBackend {
    pool: DbPool,
}

impl AuthBackend {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl axum_login::AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AppError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|err| AppError::Internal(format!("db: {err}")))?;
            let Some(user) = User::from_email(&creds.email, &mut conn)? else {
                return Ok(None);
            };
            if !user.verify_password(&creds.password) {
                return Ok(None);
            }
            Ok(Some(user))
        })
        .await
        .map_err(|err| AppError::Internal(format!("join: {err}")))?
    }

    async fn get_user(
        &self,
        user_id: &UserRef,
    ) -> Result<Option<Self::User>, Self::Error> {
        let pool = self.pool.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|err| AppError::Internal(format!("db: {err}")))?;
            User::from_ref(&user_id, &mut conn)
        })
        .await
        .map_err(|err| AppError::Internal(format!("join: {err}")))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseLocation;
    use crate::database::run_migrations;
    use crate::domain::user::NewUser;
    use axum_login::AuthnBackend;

    fn make_pool_with_user() -> (DbPool, User) {
        let pool = DatabaseLocation::Memory.connect();
        run_migrations(&pool);
        let mut conn = pool.get().unwrap();
        let user = NewUser::new("alice@example.com", "Alice", "password123")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        (pool, user)
    }

    #[tokio::test]
    async fn authenticate_with_correct_password_returns_user() {
        let (pool, user) = make_pool_with_user();
        let backend = AuthBackend::new(pool);

        let result = backend
            .authenticate(Credentials {
                email: "alice@example.com".into(),
                password: "password123".into(),
            })
            .await
            .expect("authenticate ok");

        assert_eq!(result.map(|u| u.id), Some(user.id));
    }

    #[tokio::test]
    async fn authenticate_with_wrong_password_returns_none() {
        let (pool, _) = make_pool_with_user();
        let backend = AuthBackend::new(pool);

        let result = backend
            .authenticate(Credentials {
                email: "alice@example.com".into(),
                password: "wrong".into(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn authenticate_unknown_email_returns_none() {
        let (pool, _) = make_pool_with_user();
        let backend = AuthBackend::new(pool);

        let result = backend
            .authenticate(Credentials {
                email: "nobody@example.com".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_user_by_id_returns_user() {
        let (pool, user) = make_pool_with_user();
        let backend = AuthBackend::new(pool);

        let result = backend.get_user(&user.id).await.unwrap();
        assert_eq!(result.map(|u| u.email), Some(user.email));
    }

    #[tokio::test]
    async fn get_user_unknown_id_returns_none() {
        let (pool, _) = make_pool_with_user();
        let backend = AuthBackend::new(pool);

        let unknown: UserRef = serde_json::from_value(serde_json::json!("nonexistent")).unwrap();
        let result = backend.get_user(&unknown).await.unwrap();
        assert!(result.is_none());
    }
}
