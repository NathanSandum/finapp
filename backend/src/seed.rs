use chrono::NaiveDateTime;
use diesel::prelude::*;
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::domain::account::{create_default_tree, Account, AccountRef};
use crate::domain::entry::{NewEntry, PostingInput};
use crate::domain::user::UserRef;
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::schema::{membership_events, sessions, users, workspace_members, workspaces};
use crate::DbPool;

pub const DEV_USER_ID: &str = "00000000-0000-0000-0000-000000000001";
pub const DEV_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000002";
pub const DEV_USER_EMAIL: &str = "dev@finapp.local";
pub const DEV_USER_DISPLAY_NAME: &str = "Dev User";

const SESSION_DURATION_HOURS: i64 = 24 * 365;

pub fn populate(db: &DbPool, config: &Config) -> Result<(), AppError> {
    let dev_token = config
        .dev_token
        .as_deref()
        .ok_or_else(|| AppError::Internal("seed requires dev_token".into()))?;

    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    if dev_user_exists(&mut conn)? {
        return Ok(());
    }

    let now = chrono::Utc::now().naive_utc();
    let user_ref = dev_user_ref();
    let workspace_ref = dev_workspace_ref();

    insert_dev_user(&user_ref, now, &mut conn)?;
    insert_dev_workspace(&workspace_ref, &user_ref, now, &mut conn)?;
    create_default_tree(&workspace_ref, &mut conn)?;
    insert_dev_session(&user_ref, dev_token, now, &mut conn)?;
    insert_fixture_entries(&workspace_ref, &mut conn)?;

    Ok(())
}

fn dev_user_ref() -> UserRef {
    serde_json::from_value(serde_json::json!(DEV_USER_ID))
        .expect("hardcoded dev user id is valid")
}

fn dev_workspace_ref() -> WorkspaceRef {
    serde_json::from_value(serde_json::json!(DEV_WORKSPACE_ID))
        .expect("hardcoded dev workspace id is valid")
}

fn dev_user_exists(conn: &mut SqliteConnection) -> Result<bool, AppError> {
    users::table
        .find(DEV_USER_ID)
        .count()
        .get_result::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|err| AppError::Internal(format!("dev user check failed: {err}")))
}

fn insert_dev_user(
    user_ref: &UserRef,
    now: NaiveDateTime,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_ref.to_string()),
            users::email.eq(DEV_USER_EMAIL),
            users::display_name.eq(DEV_USER_DISPLAY_NAME),
            users::password_hash.eq("dev-mode-no-password"),
            users::created_at.eq(now),
            users::updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(|err| AppError::Internal(format!("dev user insert failed: {err}")))?;
    Ok(())
}

fn insert_dev_workspace(
    workspace_ref: &WorkspaceRef,
    creator_ref: &UserRef,
    now: NaiveDateTime,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    diesel::insert_into(workspaces::table)
        .values((
            workspaces::id.eq(workspace_ref.to_string()),
            workspaces::name.eq("Dev Workspace"),
            workspaces::created_at.eq(now),
            workspaces::updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(|err| AppError::Internal(format!("dev workspace insert failed: {err}")))?;

    diesel::insert_into(workspace_members::table)
        .values((
            workspace_members::workspace_id.eq(workspace_ref.to_string()),
            workspace_members::user_id.eq(creator_ref.to_string()),
            workspace_members::role.eq(Role::Owner.as_str()),
            workspace_members::joined_at.eq(now),
        ))
        .execute(conn)
        .map_err(|err| AppError::Internal(format!("dev membership insert failed: {err}")))?;

    diesel::insert_into(membership_events::table)
        .values((
            membership_events::id.eq(uuid::Uuid::new_v4().to_string()),
            membership_events::workspace_id.eq(workspace_ref.to_string()),
            membership_events::user_id.eq(creator_ref.to_string()),
            membership_events::actor_id.eq(creator_ref.to_string()),
            membership_events::event_type.eq("owner_assigned"),
            membership_events::role.eq(Some(Role::Owner.as_str())),
            membership_events::created_at.eq(now),
        ))
        .execute(conn)
        .map_err(|err| AppError::Internal(format!("dev membership event insert failed: {err}")))?;

    Ok(())
}

fn insert_dev_session(
    user_ref: &UserRef,
    dev_token: &str,
    now: NaiveDateTime,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    let token_hash = hash_token(dev_token);
    let expires_at = now + chrono::Duration::hours(SESSION_DURATION_HOURS);

    diesel::insert_into(sessions::table)
        .values((
            sessions::id.eq(uuid::Uuid::new_v4().to_string()),
            sessions::user_id.eq(user_ref.to_string()),
            sessions::token_hash.eq(token_hash),
            sessions::expires_at.eq(expires_at),
            sessions::created_at.eq(now),
        ))
        .execute(conn)
        .map_err(|err| AppError::Internal(format!("dev session insert failed: {err}")))?;
    Ok(())
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn account_by_name(
    workspace_ref: &WorkspaceRef,
    name: &str,
    conn: &mut SqliteConnection,
) -> Result<AccountRef, AppError> {
    use crate::schema::accounts;
    accounts::table
        .filter(accounts::workspace_id.eq(workspace_ref))
        .filter(accounts::name.eq(name))
        .select(Account::as_select())
        .first(conn)
        .map(|a| a.id)
        .map_err(|err| AppError::Internal(format!("account '{name}' lookup failed: {err}")))
}

fn insert_fixture_entries(
    workspace_ref: &WorkspaceRef,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    let chequing = account_by_name(workspace_ref, "Chequing", conn)?;
    let savings = account_by_name(workspace_ref, "Savings", conn)?;
    let credit_card = account_by_name(workspace_ref, "Credit Card", conn)?;
    let employment = account_by_name(workspace_ref, "Employment", conn)?;
    let rent = account_by_name(workspace_ref, "Rent", conn)?;
    let groceries = account_by_name(workspace_ref, "Groceries", conn)?;
    let dining = account_by_name(workspace_ref, "Dining", conn)?;
    let transit = account_by_name(workspace_ref, "Transit", conn)?;
    let utilities = account_by_name(workspace_ref, "Utilities", conn)?;
    let entertainment = account_by_name(workspace_ref, "Entertainment", conn)?;

    let entries: Vec<(&str, &str, Vec<PostingInput>)> = vec![
        (
            "2026-05-01",
            "Salary — May",
            vec![
                PostingInput::new(chequing.clone(), 540000),
                PostingInput::new(employment.clone(), -540000),
            ],
        ),
        (
            "2026-05-01",
            "Rent — May",
            vec![
                PostingInput::new(rent.clone(), 180000),
                PostingInput::new(chequing.clone(), -180000),
            ],
        ),
        (
            "2026-05-02",
            "Whole Foods",
            vec![
                PostingInput::new(groceries.clone(), 8742).note("weekly shop"),
                PostingInput::new(credit_card.clone(), -8742),
            ],
        ),
        (
            "2026-05-03",
            "Subway pass",
            vec![
                PostingInput::new(transit.clone(), 13500),
                PostingInput::new(chequing.clone(), -13500),
            ],
        ),
        (
            "2026-05-04",
            "Coffee",
            vec![
                PostingInput::new(dining.clone(), 425),
                PostingInput::new(credit_card.clone(), -425),
            ],
        ),
        (
            "2026-05-05",
            "Hydro bill",
            vec![
                PostingInput::new(utilities.clone(), 7820),
                PostingInput::new(chequing.clone(), -7820),
            ],
        ),
        (
            "2026-05-08",
            "Movie night",
            vec![
                PostingInput::new(entertainment.clone(), 4400),
                PostingInput::new(credit_card.clone(), -4400),
            ],
        ),
        (
            "2026-05-10",
            "Trader Joe's",
            vec![
                PostingInput::new(groceries.clone(), 6235),
                PostingInput::new(credit_card.clone(), -6235),
            ],
        ),
        (
            "2026-05-12",
            "Transfer to savings",
            vec![
                PostingInput::new(savings.clone(), 50000),
                PostingInput::new(chequing.clone(), -50000),
            ],
        ),
        (
            "2026-05-15",
            "Credit card payment",
            vec![
                PostingInput::new(credit_card.clone(), 25000),
                PostingInput::new(chequing.clone(), -25000),
            ],
        ),
        (
            "2026-05-18",
            "Dinner — Trattoria",
            vec![
                PostingInput::new(dining.clone(), 9450),
                PostingInput::new(credit_card.clone(), -9450),
            ],
        ),
        (
            "2026-05-22",
            "Costco",
            vec![
                PostingInput::new(groceries.clone(), 14200),
                PostingInput::new(credit_card.clone(), -14200),
            ],
        ),
    ];

    for (date, description, postings) in entries {
        NewEntry::new(workspace_ref.clone(), date, description, postings).insert(conn)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseLocation, Mode};
    use crate::domain::session::Session;
    use diesel_migrations::MigrationHarness;

    fn dev_config() -> Config {
        Config {
            mode: Mode::Dev,
            database: DatabaseLocation::Memory,
            port: 3000,
            dev_token: Some("dev-token-secret".into()),
        }
    }

    fn populate_via_db(config: &Config) -> DbPool {
        let db = config.database.connect();
        {
            let mut conn = db.get().expect("get connection");
            conn.run_pending_migrations(crate::database::MIGRATIONS)
                .expect("run migrations");
        }
        populate(&db, config).expect("seed populate succeeds");
        db
    }

    #[test]
    fn populate_creates_dev_user() {
        let db = populate_via_db(&dev_config());
        let mut conn = db.get().unwrap();
        let count: i64 = users::table
            .find(DEV_USER_ID)
            .count()
            .get_result(&mut conn)
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn populate_creates_dev_workspace_with_owner_membership() {
        let db = populate_via_db(&dev_config());
        let mut conn = db.get().unwrap();

        let workspace_count: i64 = workspaces::table
            .find(DEV_WORKSPACE_ID)
            .count()
            .get_result(&mut conn)
            .unwrap();
        assert_eq!(workspace_count, 1);

        let role: String = workspace_members::table
            .filter(workspace_members::workspace_id.eq(DEV_WORKSPACE_ID))
            .filter(workspace_members::user_id.eq(DEV_USER_ID))
            .select(workspace_members::role)
            .first(&mut conn)
            .unwrap();
        assert_eq!(role, Role::Owner.as_str());
    }

    #[test]
    fn populate_creates_default_account_tree() {
        use crate::schema::accounts;
        let db = populate_via_db(&dev_config());
        let mut conn = db.get().unwrap();

        let count: i64 = accounts::table
            .filter(accounts::workspace_id.eq(DEV_WORKSPACE_ID))
            .count()
            .get_result(&mut conn)
            .unwrap();
        assert!(count >= 5, "default tree should have at least 5 accounts");
    }

    #[test]
    fn dev_token_validates_to_dev_user_session() {
        let config = dev_config();
        let db = populate_via_db(&config);
        let mut conn = db.get().unwrap();

        let user = Session::validate(config.dev_token.as_deref().unwrap(), &mut conn)
            .expect("session validates")
            .expect("dev token resolves to a user");

        assert_eq!(user.id.to_string(), DEV_USER_ID);
    }

    #[test]
    fn populate_creates_fixture_entries() {
        use crate::schema::entries;
        let db = populate_via_db(&dev_config());
        let mut conn = db.get().unwrap();
        let count: i64 = entries::table
            .filter(entries::workspace_id.eq(DEV_WORKSPACE_ID))
            .count()
            .get_result(&mut conn)
            .unwrap();
        assert!(count >= 100, "expected rich fixture set, got {count}");
    }

    #[test]
    fn fixture_entries_span_at_least_twelve_months() {
        use crate::schema::entries;
        let db = populate_via_db(&dev_config());
        let mut conn = db.get().unwrap();

        let dates: Vec<String> = entries::table
            .filter(entries::workspace_id.eq(DEV_WORKSPACE_ID))
            .select(entries::date)
            .load(&mut conn)
            .unwrap();

        let mut months = std::collections::BTreeSet::new();
        for date in &dates {
            months.insert(date[..7].to_string());
        }
        assert!(
            months.len() >= 12,
            "fixture should cover at least 12 distinct months, got {}: {:?}",
            months.len(),
            months
        );
    }

    #[test]
    fn fixture_entries_touch_investment_and_opening_accounts() {
        use crate::schema::{accounts, entries, postings};
        let db = populate_via_db(&dev_config());
        let mut conn = db.get().unwrap();

        for name in ["TFSA", "RRSP", "Opening Balances"] {
            let count: i64 = postings::table
                .inner_join(accounts::table.on(postings::account_id.eq(accounts::id)))
                .inner_join(entries::table.on(postings::entry_id.eq(entries::id)))
                .filter(entries::workspace_id.eq(DEV_WORKSPACE_ID))
                .filter(accounts::name.eq(name))
                .count()
                .get_result(&mut conn)
                .unwrap();
            assert!(count > 0, "expected fixture postings touching '{name}'");
        }
    }

    #[test]
    fn populate_is_idempotent() {
        let config = dev_config();
        let db = populate_via_db(&config);
        // Second call must not error or duplicate.
        populate(&db, &config).expect("second populate succeeds");

        let mut conn = db.get().unwrap();
        let user_count: i64 = users::table.count().get_result(&mut conn).unwrap();
        assert_eq!(user_count, 1);
    }

}
