use chrono::{Datelike, NaiveDate, NaiveDateTime};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::account::{Account, AccountRef};
use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::baselines;

// ── PeriodType ──────────────────────────────────────────────────────────────

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    Monthly,
}

impl PeriodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeriodType::Monthly => "monthly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "monthly" => Some(PeriodType::Monthly),
            _ => None,
        }
    }
}

impl FromSql<Text, Sqlite> for PeriodType {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        PeriodType::from_str(&s).ok_or_else(|| format!("unknown period type: {s}").into())
    }
}

impl ToSql<Text, Sqlite> for PeriodType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(self.as_str(), out)
    }
}


// ── BaselineRef ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct BaselineRef(String);

impl std::fmt::Display for BaselineRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Baseline ────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, Queryable, Selectable)]
#[diesel(table_name = baselines)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Baseline {
    pub id: BaselineRef,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    #[diesel(column_name = account_id)]
    pub account: AccountRef,
    pub period_type: PeriodType,
    pub period_start: String,
    pub amount: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Baseline {
    pub fn from_ref(
        baseline_ref: &BaselineRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Baseline>, AppError> {
        baselines::table
            .find(baseline_ref)
            .select(Baseline::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("baseline lookup failed: {err}")))
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        diesel::delete(baselines::table.find(&self.id))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("baseline delete failed: {err}")))?;
        Ok(())
    }

    /// Variance = actual spending - baseline amount.
    /// Positive = over budget, negative = under budget.
    pub fn variance(&self, conn: &mut SqliteConnection) -> Result<i64, AppError> {
        let account = Account::from_ref(&self.account, conn)?
            .ok_or_else(|| AppError::Internal("baseline account not found".to_string()))?;
        let period_start = NaiveDate::parse_from_str(&self.period_start, "%Y-%m-%d")
            .map_err(|err| AppError::Internal(format!("invalid period_start date: {err}")))?;
        let period_end = end_of_month_date(period_start)?;
        let actual =
            account.balance_between_with_descendants(period_start, period_end, conn)?;
        Ok(actual - self.amount as i64)
    }
}

// ── Query methods on owning types ────────────────────────────────────────────

impl AccountRef {
    pub fn baselines(&self, conn: &mut SqliteConnection) -> Result<Vec<Baseline>, AppError> {
        baselines::table
            .filter(baselines::account_id.eq(self))
            .select(Baseline::as_select())
            .order(baselines::period_start.desc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("account baselines query failed: {err}")))
    }
}

impl WorkspaceRef {
    pub fn baselines(&self, conn: &mut SqliteConnection) -> Result<Vec<Baseline>, AppError> {
        baselines::table
            .filter(baselines::workspace_id.eq(self))
            .select(Baseline::as_select())
            .order(baselines::period_start.desc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("workspace baselines query failed: {err}")))
    }
}

// ── NewBaseline ─────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = baselines)]
struct NewBaselineRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    #[diesel(column_name = account_id)]
    account: AccountRef,
    period_type: PeriodType,
    period_start: String,
    amount: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewBaseline {
    workspace: WorkspaceRef,
    account: AccountRef,
    period_type: PeriodType,
    period_start: String,
    amount: i32,
}

impl NewBaseline {
    pub fn monthly(
        workspace: WorkspaceRef,
        account: AccountRef,
        period_start: &str,
        amount: i32,
    ) -> Self {
        NewBaseline {
            workspace,
            account,
            period_type: PeriodType::Monthly,
            period_start: period_start.to_string(),
            amount,
        }
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Baseline, AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::insert_into(baselines::table)
            .values(NewBaselineRow {
                id: Uuid::new_v4().to_string(),
                workspace: self.workspace,
                account: self.account,
                period_type: self.period_type,
                period_start: self.period_start,
                amount: self.amount,
                created_at: now,
                updated_at: now,
            })
            .returning(Baseline::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => AppError::Conflict(
                    "baseline already exists for this account and period".to_string(),
                ),
                other => AppError::Internal(format!("baseline insert failed: {other}")),
            })
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn end_of_month_date(date: NaiveDate) -> Result<NaiveDate, AppError> {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let first_of_next = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::Internal("invalid date computation".to_string()))?;
    first_of_next
        .pred_opt()
        .ok_or_else(|| AppError::Internal("date underflow".to_string()))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::Account;
    use crate::domain::entry::{NewEntry, PostingInput};
    use crate::domain::user::NewUser;
    use crate::domain::workspace::{NewWorkspace, Workspace};
    use crate::test_helpers::setup_test_db;

    fn setup() -> (SqliteConnection, Workspace) {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace = NewWorkspace::new("My Finances", user.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        (conn, workspace)
    }

    fn find_account(name: &str, workspace: &Workspace, conn: &mut SqliteConnection) -> Account {
        let all = workspace.as_ref().accounts(conn).unwrap();
        all.into_iter().find(|a| a.name == name).unwrap()
    }

    fn make_entry(
        workspace: &Workspace,
        from: &Account,
        to: &Account,
        amount: i32,
        date: &str,
        conn: &mut SqliteConnection,
    ) {
        NewEntry::new(
            workspace.as_ref().clone(),
            date.parse::<NaiveDate>().unwrap(),
            "test",
            vec![
                PostingInput::new(from.id.clone(), -amount),
                PostingInput::new(to.id.clone(), amount),
            ],
        )
        .insert(conn)
        .unwrap();
    }

    fn nd(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    // ── end_of_month_date tests ────────────────────────────────────

    #[test]
    fn end_of_month_correctness() {
        assert_eq!(end_of_month_date(nd("2026-01-01")).unwrap(), nd("2026-01-31"));
        assert_eq!(end_of_month_date(nd("2026-02-01")).unwrap(), nd("2026-02-28"));
        assert_eq!(end_of_month_date(nd("2024-02-01")).unwrap(), nd("2024-02-29"));
        assert_eq!(end_of_month_date(nd("2026-12-01")).unwrap(), nd("2026-12-31"));
        assert_eq!(end_of_month_date(nd("2026-04-15")).unwrap(), nd("2026-04-30"));
    }

    // ── Baseline CRUD tests ────────────────────────────────────────

    #[test]
    fn create_baseline_succeeds() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        let baseline = NewBaseline::monthly(
            workspace.as_ref().clone(),
            food.id.clone(),
            "2026-01-01",
            50000,
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(baseline.amount, 50000);
        assert_eq!(baseline.period_start, "2026-01-01");
        assert_eq!(baseline.period_type, PeriodType::Monthly);
        assert_eq!(baseline.account, food.id);
    }

    #[test]
    fn duplicate_baseline_rejected() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        NewBaseline::monthly(workspace.as_ref().clone(), food.id.clone(), "2026-01-01", 50000)
            .insert(&mut conn)
            .unwrap();

        let err = NewBaseline::monthly(
            workspace.as_ref().clone(),
            food.id.clone(),
            "2026-01-01",
            60000,
        )
        .insert(&mut conn)
        .expect_err("duplicate should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn baselines_for_account_returns_matching() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        NewBaseline::monthly(workspace.as_ref().clone(), food.id.clone(), "2026-01-01", 50000)
            .insert(&mut conn)
            .unwrap();
        NewBaseline::monthly(workspace.as_ref().clone(), rent.id.clone(), "2026-01-01", 150000)
            .insert(&mut conn)
            .unwrap();

        let food_baselines = food.id.baselines(&mut conn).unwrap();
        assert_eq!(food_baselines.len(), 1);
        assert_eq!(food_baselines[0].amount, 50000);
    }

    #[test]
    fn baselines_for_workspace_returns_all() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        NewBaseline::monthly(workspace.as_ref().clone(), food.id.clone(), "2026-01-01", 50000)
            .insert(&mut conn)
            .unwrap();
        NewBaseline::monthly(workspace.as_ref().clone(), rent.id.clone(), "2026-01-01", 150000)
            .insert(&mut conn)
            .unwrap();

        let all = workspace.as_ref().baselines(&mut conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    // ── Variance tests ─────────────────────────────────────────────

    #[test]
    fn variance_no_entries_returns_negative() {
        let (mut conn, workspace) = setup();
        let food = find_account("Food", &workspace, &mut conn);

        let baseline = NewBaseline::monthly(
            workspace.as_ref().clone(),
            food.id.clone(),
            "2026-01-01",
            10000,
        )
        .insert(&mut conn)
        .unwrap();

        // No entries, actual = 0, variance = 0 - 10000 = -10000
        assert_eq!(baseline.variance(&mut conn).unwrap(), -10000);
    }

    #[test]
    fn variance_exact_match_returns_zero() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 5000, "2026-01-15", &mut conn);

        let baseline = NewBaseline::monthly(
            workspace.as_ref().clone(),
            food.id.clone(),
            "2026-01-01",
            5000,
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(baseline.variance(&mut conn).unwrap(), 0);
    }

    #[test]
    fn variance_over_budget_returns_positive() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 7500, "2026-01-15", &mut conn);

        let baseline = NewBaseline::monthly(
            workspace.as_ref().clone(),
            food.id.clone(),
            "2026-01-01",
            5000,
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(baseline.variance(&mut conn).unwrap(), 2500);
    }

    #[test]
    fn variance_respects_period_boundaries() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        // Jan entry
        make_entry(&workspace, &chequing, &food, 3000, "2026-01-15", &mut conn);
        // Feb entry — should NOT count for Jan baseline
        make_entry(&workspace, &chequing, &food, 2000, "2026-02-10", &mut conn);

        let baseline = NewBaseline::monthly(
            workspace.as_ref().clone(),
            food.id.clone(),
            "2026-01-01",
            5000,
        )
        .insert(&mut conn)
        .unwrap();

        // Only Jan spending (3000), variance = 3000 - 5000 = -2000
        assert_eq!(baseline.variance(&mut conn).unwrap(), -2000);
    }
}
