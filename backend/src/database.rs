use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::DbPool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(db: &DbPool) {
    let mut conn = db
        .get()
        .expect("failed to acquire connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
}
