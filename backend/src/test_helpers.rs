use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub fn setup_test_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:")
        .expect("failed to create in-memory SQLite connection");

    diesel::sql_query(include_str!("../migrations/0001_initial.sql"))
        .execute(&mut conn)
        .expect("failed to run initial migration");

    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_run_successfully() {
        let mut conn = setup_test_db();
        // Verify the connection is usable after migrations.
        diesel::sql_query("SELECT 1")
            .execute(&mut conn)
            .expect("connection should be queryable after migrations");
    }
}
