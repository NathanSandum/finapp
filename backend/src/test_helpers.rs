use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::MigrationHarness;

use crate::database::MIGRATIONS;

pub fn setup_test_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:")
        .expect("failed to create in-memory SQLite connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_returns_usable_connection() {
        let mut conn = setup_test_db();
        diesel::sql_query("SELECT 1")
            .execute(&mut conn)
            .expect("connection should be queryable after setup");
    }

    #[test]
    fn each_call_returns_isolated_database() {
        // In-memory SQLite connections are fully isolated from each other.
        // Writing to one must not affect the other.
        let mut conn_a = setup_test_db();
        let mut conn_b = setup_test_db();

        diesel::sql_query(
            "INSERT INTO users (id, created_at) VALUES ('test-user', datetime('now'))",
        )
        .execute(&mut conn_a)
        .expect("insert into conn_a should succeed");

        let count: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM users")
            .load::<CountRow>(&mut conn_b)
            .expect("query conn_b should succeed")
            .into_iter()
            .next()
            .map(|r| r.count)
            .unwrap_or(0);

        assert_eq!(count, 0, "conn_b should not see data written to conn_a");
    }

    #[test]
    fn all_migrations_applied() {
        let mut conn = setup_test_db();
        // Both placeholder tables from the initial migration must exist.
        diesel::sql_query("SELECT id FROM users LIMIT 1")
            .execute(&mut conn)
            .expect("users table should exist after migrations");
        diesel::sql_query("SELECT id FROM workspaces LIMIT 1")
            .execute(&mut conn)
            .expect("workspaces table should exist after migrations");
    }
}

// Helper for COUNT(*) queries in tests.
#[derive(diesel::QueryableByName)]
#[cfg(test)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}
