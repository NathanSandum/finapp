use chrono::NaiveDateTime;
use diesel::prelude::*;
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::domain::account::{create_default_tree, Account, SystemAccount};
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

fn insert_fixture_entries(
    workspace_ref: &WorkspaceRef,
    conn: &mut SqliteConnection,
) -> Result<(), AppError> {
    let chequing = Account::from_system(workspace_ref, SystemAccount::Chequing, conn)?.id;
    let opening = Account::from_system(workspace_ref, SystemAccount::OpeningBalances, conn)?.id;
    let savings = Account::from_system(workspace_ref, SystemAccount::Savings, conn)?.id;
    let tfsa = Account::from_custom_name(workspace_ref, "TFSA", conn)?.id;
    let rrsp = Account::from_custom_name(workspace_ref, "RRSP", conn)?.id;
    let credit_card = Account::from_custom_name(workspace_ref, "Credit Card", conn)?.id;
    let employment = Account::from_custom_name(workspace_ref, "Employment", conn)?.id;
    let rent = Account::from_custom_name(workspace_ref, "Rent", conn)?.id;
    let groceries = Account::from_custom_name(workspace_ref, "Groceries", conn)?.id;
    let dining = Account::from_custom_name(workspace_ref, "Dining", conn)?.id;
    let transit = Account::from_custom_name(workspace_ref, "Transit", conn)?.id;
    let utilities = Account::from_custom_name(workspace_ref, "Utilities", conn)?.id;
    let entertainment = Account::from_custom_name(workspace_ref, "Entertainment", conn)?.id;

    let mut entries: Vec<(String, String, Vec<PostingInput>)> = Vec::new();

    let opening_chequing: i32 = 500_000;
    let opening_savings: i32 = 1_000_000;
    let opening_tfsa: i32 = 2_000_000;
    let opening_rrsp: i32 = 3_000_000;
    let opening_credit_card: i32 = -120_000;
    let opening_offset: i32 = -(opening_chequing
        + opening_savings
        + opening_tfsa
        + opening_rrsp
        + opening_credit_card);

    entries.push((
        "2025-04-30".to_string(),
        "Opening balances".to_string(),
        vec![
            PostingInput::new(chequing.clone(), opening_chequing),
            PostingInput::new(savings.clone(), opening_savings),
            PostingInput::new(tfsa.clone(), opening_tfsa),
            PostingInput::new(rrsp.clone(), opening_rrsp),
            PostingInput::new(credit_card.clone(), opening_credit_card),
            PostingInput::new(opening.clone(), opening_offset),
        ],
    ));

    let months: [(i32, u32); 13] = [
        (2025, 5),
        (2025, 6),
        (2025, 7),
        (2025, 8),
        (2025, 9),
        (2025, 10),
        (2025, 11),
        (2025, 12),
        (2026, 1),
        (2026, 2),
        (2026, 3),
        (2026, 4),
        (2026, 5),
    ];

    for &(year, month) in months.iter() {
        let raise = if year == 2026 { 20_000 } else { 0 };
        let bonus = if year == 2025 && month == 12 { 150_000 } else { 0 };
        let salary = 540_000 + raise + bonus;
        entries.push((
            format!("{year:04}-{month:02}-01"),
            "Salary".to_string(),
            vec![
                PostingInput::new(chequing.clone(), salary),
                PostingInput::new(employment.clone(), -salary),
            ],
        ));

        entries.push((
            format!("{year:04}-{month:02}-01"),
            "Rent".to_string(),
            vec![
                PostingInput::new(rent.clone(), 180_000),
                PostingInput::new(chequing.clone(), -180_000),
            ],
        ));

        entries.push((
            format!("{year:04}-{month:02}-03"),
            "Transit pass".to_string(),
            vec![
                PostingInput::new(transit.clone(), 13_500),
                PostingInput::new(chequing.clone(), -13_500),
            ],
        ));

        let hydro = match month {
            12 | 1 | 2 => 11_500,
            3 | 11 => 9_200,
            6 | 7 | 8 => 6_400,
            _ => 7_800,
        };
        entries.push((
            format!("{year:04}-{month:02}-05"),
            "Hydro".to_string(),
            vec![
                PostingInput::new(utilities.clone(), hydro),
                PostingInput::new(chequing.clone(), -hydro),
            ],
        ));

        entries.push((
            format!("{year:04}-{month:02}-12"),
            "Internet & phone".to_string(),
            vec![
                PostingInput::new(utilities.clone(), 14_500),
                PostingInput::new(chequing.clone(), -14_500),
            ],
        ));

        let grocery_runs: [(u32, i32, &str); 4] = [
            (7, 8_742, "Whole Foods"),
            (14, 6_235, "Trader Joe's"),
            (21, 14_200, "Costco"),
            (27, 7_860, "Loblaws"),
        ];
        for (day, amount, vendor) in grocery_runs {
            entries.push((
                format!("{year:04}-{month:02}-{day:02}"),
                vendor.to_string(),
                vec![
                    PostingInput::new(groceries.clone(), amount),
                    PostingInput::new(credit_card.clone(), -amount),
                ],
            ));
        }

        let dining_runs: &[(u32, i32, &str)] = match month {
            6 | 7 | 8 => &[
                (8, 4_250, "Patio coffee"),
                (16, 6_800, "Brunch"),
                (22, 9_450, "Trattoria"),
                (26, 5_200, "Sushi takeout"),
            ],
            _ => &[(8, 4_250, "Coffee"), (22, 9_450, "Trattoria")],
        };
        for &(day, amount, vendor) in dining_runs {
            entries.push((
                format!("{year:04}-{month:02}-{day:02}"),
                vendor.to_string(),
                vec![
                    PostingInput::new(dining.clone(), amount),
                    PostingInput::new(credit_card.clone(), -amount),
                ],
            ));
        }

        let entertainment_amount = if month % 3 == 0 { 9_800 } else { 4_400 };
        let entertainment_label = if entertainment_amount > 5_000 {
            "Concert"
        } else {
            "Movie night"
        };
        entries.push((
            format!("{year:04}-{month:02}-19"),
            entertainment_label.to_string(),
            vec![
                PostingInput::new(entertainment.clone(), entertainment_amount),
                PostingInput::new(credit_card.clone(), -entertainment_amount),
            ],
        ));

        entries.push((
            format!("{year:04}-{month:02}-15"),
            "Transfer to savings".to_string(),
            vec![
                PostingInput::new(savings.clone(), 50_000),
                PostingInput::new(chequing.clone(), -50_000),
            ],
        ));
        entries.push((
            format!("{year:04}-{month:02}-15"),
            "TFSA contribution".to_string(),
            vec![
                PostingInput::new(tfsa.clone(), 30_000),
                PostingInput::new(chequing.clone(), -30_000),
            ],
        ));
        entries.push((
            format!("{year:04}-{month:02}-15"),
            "RRSP contribution".to_string(),
            vec![
                PostingInput::new(rrsp.clone(), 40_000),
                PostingInput::new(chequing.clone(), -40_000),
            ],
        ));

        if year == 2025 && month == 12 {
            entries.push((
                format!("{year:04}-{month:02}-28"),
                "Year-end RRSP top-up".to_string(),
                vec![
                    PostingInput::new(rrsp.clone(), 200_000),
                    PostingInput::new(chequing.clone(), -200_000),
                ],
            ));
        }

        let cc_payment = match month {
            6 | 7 | 8 => 50_000,
            _ => 35_000,
        };
        entries.push((
            format!("{year:04}-{month:02}-25"),
            "Credit card payment".to_string(),
            vec![
                PostingInput::new(credit_card.clone(), cc_payment),
                PostingInput::new(chequing.clone(), -cc_payment),
            ],
        ));
    }

    for (date, description, postings) in entries {
        NewEntry::new(workspace_ref.clone(), &date, &description, postings).insert(conn)?;
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

        // Custom by name
        for name in ["TFSA", "RRSP"] {
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

        // System OpeningBalances by account_type tag
        let count: i64 = postings::table
            .inner_join(accounts::table.on(postings::account_id.eq(accounts::id)))
            .inner_join(entries::table.on(postings::entry_id.eq(entries::id)))
            .filter(entries::workspace_id.eq(DEV_WORKSPACE_ID))
            .filter(accounts::account_type.eq("OpeningBalances"))
            .count()
            .get_result(&mut conn)
            .unwrap();
        assert!(count > 0, "expected fixture postings touching OpeningBalances");
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
