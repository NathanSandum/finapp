use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::{Config, Mode};
use crate::DbPool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn build(config: &Config) -> DbPool {
    let pool = config.database.connect();
    if config.mode == Mode::Dev {
        run_migrations(&pool);
    }
    pool
}

fn run_migrations(pool: &DbPool) {
    let mut conn = pool
        .get()
        .expect("failed to acquire connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseLocation;
    use diesel::prelude::*;

    #[test]
    fn dev_build_runs_migrations() {
        let config = Config {
            mode: Mode::Dev,
            database: DatabaseLocation::Memory,
            port: 3000,
            dev_token: Some("dev".into()),
        };
        let pool = build(&config);
        let mut conn = pool.get().expect("get connection");
        diesel::sql_query("SELECT id FROM users LIMIT 1")
            .execute(&mut conn)
            .expect("users table should exist after dev build");
    }

    #[test]
    fn prod_build_does_not_run_migrations() {
        // Prod skips auto-migrate. Memory db has no schema yet, so query should fail.
        // Must use Dev mode + memory location to bypass validation, then manually
        // assert behavior — but config validation forbids Prod+Memory. Instead,
        // run a temp file db for prod and rely on absence of schema as proof.
        let temp_path = std::env::temp_dir().join(format!(
            "finapp-test-{}.db",
            uuid::Uuid::new_v4()
        ));
        let config = Config {
            mode: Mode::Prod,
            database: DatabaseLocation::File(temp_path.clone()),
            port: 3000,
            dev_token: None,
        };
        let pool = build(&config);
        let mut conn = pool.get().expect("get connection");
        let result =
            diesel::sql_query("SELECT id FROM users LIMIT 1").execute(&mut conn);
        assert!(
            result.is_err(),
            "prod build should not auto-migrate: users table should not exist"
        );
        let _ = std::fs::remove_file(&temp_path);
    }
}
