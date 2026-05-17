use async_trait::async_trait;
use axum_login::tower_sessions::session::{Id, Record};
use axum_login::tower_sessions::session_store::{self, SessionStore};
use diesel::prelude::*;
use time::OffsetDateTime;

use crate::schema::auth_sessions;
use crate::DbPool;

#[derive(Clone, Debug)]
pub struct DieselSessionStore {
    pool: DbPool,
}

impl DieselSessionStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = auth_sessions)]
struct SessionRow {
    id: String,
    data: Vec<u8>,
    expiry_date: i64,
}

fn encode_record(record: &Record) -> session_store::Result<Vec<u8>> {
    serde_json::to_vec(record).map_err(|err| session_store::Error::Encode(err.to_string()))
}

fn decode_record(bytes: &[u8]) -> session_store::Result<Record> {
    serde_json::from_slice(bytes).map_err(|err| session_store::Error::Decode(err.to_string()))
}

fn backend_error<E: std::fmt::Display>(err: E) -> session_store::Error {
    session_store::Error::Backend(err.to_string())
}

async fn run_blocking<F, T>(pool: DbPool, f: F) -> session_store::Result<T>
where
    F: FnOnce(&mut SqliteConnection) -> session_store::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(backend_error)?;
        f(&mut conn)
    })
    .await
    .map_err(backend_error)?
}

#[async_trait]
impl SessionStore for DieselSessionStore {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let row = SessionRow {
            id: record.id.to_string(),
            data: encode_record(record)?,
            expiry_date: record.expiry_date.unix_timestamp(),
        };
        run_blocking(self.pool.clone(), move |conn| {
            diesel::replace_into(auth_sessions::table)
                .values(&row)
                .execute(conn)
                .map_err(backend_error)?;
            Ok(())
        })
        .await
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id = session_id.to_string();
        run_blocking(self.pool.clone(), move |conn| {
            let row: Option<(Vec<u8>, i64)> = auth_sessions::table
                .find(&id)
                .select((auth_sessions::data, auth_sessions::expiry_date))
                .first(conn)
                .optional()
                .map_err(backend_error)?;

            let Some((data, expiry)) = row else {
                return Ok(None);
            };

            if expiry <= OffsetDateTime::now_utc().unix_timestamp() {
                return Ok(None);
            }

            Ok(Some(decode_record(&data)?))
        })
        .await
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let id = session_id.to_string();
        run_blocking(self.pool.clone(), move |conn| {
            diesel::delete(auth_sessions::table.find(&id))
                .execute(conn)
                .map_err(backend_error)?;
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseLocation;
    use crate::database::run_migrations;
    use axum_login::tower_sessions::session::Record;
    use std::collections::HashMap;
    use time::Duration;

    fn make_pool() -> DbPool {
        let pool = DatabaseLocation::Memory.connect();
        run_migrations(&pool);
        pool
    }

    fn make_record() -> Record {
        Record {
            id: Id::default(),
            data: HashMap::new(),
            expiry_date: OffsetDateTime::now_utc() + Duration::hours(1),
        }
    }

    #[tokio::test]
    async fn save_then_load_round_trips() {
        let store = DieselSessionStore::new(make_pool());
        let record = make_record();

        store.save(&record).await.expect("save");
        let loaded = store.load(&record.id).await.expect("load").expect("present");

        assert_eq!(loaded.id, record.id);
        assert_eq!(loaded.expiry_date, record.expiry_date);
    }

    #[tokio::test]
    async fn load_missing_returns_none() {
        let store = DieselSessionStore::new(make_pool());
        let loaded = store.load(&Id::default()).await.expect("load");
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn delete_removes_record() {
        let store = DieselSessionStore::new(make_pool());
        let record = make_record();
        store.save(&record).await.unwrap();

        store.delete(&record.id).await.expect("delete");
        let loaded = store.load(&record.id).await.expect("load");
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn save_overwrites_existing_record() {
        let store = DieselSessionStore::new(make_pool());
        let mut record = make_record();
        store.save(&record).await.unwrap();

        record.expiry_date = OffsetDateTime::now_utc() + Duration::hours(2);
        store.save(&record).await.expect("overwrite");

        let loaded = store.load(&record.id).await.unwrap().unwrap();
        assert_eq!(loaded.expiry_date, record.expiry_date);
    }

    #[tokio::test]
    async fn load_expired_returns_none() {
        let store = DieselSessionStore::new(make_pool());
        let record = Record {
            id: Id::default(),
            data: HashMap::new(),
            expiry_date: OffsetDateTime::now_utc() - Duration::hours(1),
        };
        store.save(&record).await.unwrap();

        let loaded = store.load(&record.id).await.expect("load");
        assert!(loaded.is_none(), "expired session should not load");
    }
}
