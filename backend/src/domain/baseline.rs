use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use uuid::Uuid;

use crate::domain::category::CategoryRef;
use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::schema::{baselines, line_items, transactions};

// ── PeriodType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PeriodType {
    Monthly,
    Quarterly,
}

impl PeriodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeriodType::Monthly => "monthly",
            PeriodType::Quarterly => "quarterly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "monthly" => Some(PeriodType::Monthly),
            "quarterly" => Some(PeriodType::Quarterly),
            _ => None,
        }
    }
}

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = baselines)]
pub struct Baseline {
    pub id: String,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    #[diesel(column_name = category_id)]
    pub category: CategoryRef,
    pub period_type: String,
    pub period_start: String,
    pub amount: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Baseline {
    pub fn update_amount(&self, amount: i32, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(baselines::table.find(&self.id))
            .set((baselines::amount.eq(amount), baselines::updated_at.eq(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("baseline update failed: {err}")))?;
        Ok(())
    }
}

impl WorkspaceRef {
    pub fn baselines(
        &self,
        period_start: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Baseline>, AppError> {
        baselines::table
            .filter(baselines::workspace_id.eq(self))
            .filter(baselines::period_start.eq(period_start))
            .select(Baseline::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("baseline list failed: {err}")))
    }

    pub fn period_summary(
        &self,
        period_start: &str,
        period_end: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<CategoryVariance>, AppError> {
        let baselines = self.baselines(period_start, conn)?;
        let actuals = actuals_by_category(self, period_start, period_end, conn)?;

        let mut results: Vec<CategoryVariance> = Vec::new();
        let mut seen_categories: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for baseline in &baselines {
            let actual = actuals
                .iter()
                .find(|(category, _)| *category == baseline.category)
                .map(|(_, amount)| *amount)
                .unwrap_or(0);

            let variance = actual - baseline.amount as i64;
            let variance_pct = if baseline.amount != 0 {
                Some((variance as f64 / baseline.amount as f64) * 100.0)
            } else if actual != 0 {
                None
            } else {
                Some(0.0)
            };

            results.push(CategoryVariance {
                category: baseline.category.to_string(),
                actual,
                baseline: Some(baseline.amount),
                variance: Some(variance),
                variance_pct,
            });
            seen_categories.insert(baseline.category.to_string());
        }

        for (category, actual) in &actuals {
            let category_str = category.to_string();
            if !seen_categories.contains(&category_str) {
                results.push(CategoryVariance {
                    category: category_str,
                    actual: *actual,
                    baseline: None,
                    variance: None,
                    variance_pct: None,
                });
            }
        }

        Ok(results)
    }
}

// ── NewBaseline ──────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = baselines)]
struct NewBaselineRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    #[diesel(column_name = category_id)]
    category: CategoryRef,
    period_type: String,
    period_start: String,
    amount: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewBaseline {
    workspace: WorkspaceRef,
    category: CategoryRef,
    period_type: PeriodType,
    period_start: String,
    amount: i32,
}

impl NewBaseline {
    pub fn new(
        workspace: &WorkspaceRef,
        category: &CategoryRef,
        period_type: PeriodType,
        period_start: &str,
        amount: i32,
    ) -> Self {
        NewBaseline {
            workspace: workspace.clone(),
            category: category.clone(),
            period_type,
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
                category: self.category,
                period_type: self.period_type.as_str().to_string(),
                period_start: self.period_start.clone(),
                amount: self.amount,
                created_at: now,
                updated_at: now,
            })
            .returning(Baseline::as_returning())
            .get_result(conn)
            .map_err(|err| match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    AppError::Conflict(format!(
                        "baseline already exists for this category and period ({})",
                        self.period_start
                    ))
                }
                other => AppError::Internal(format!("baseline insert failed: {other}")),
            })
    }
}

// ── Variance ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CategoryVariance {
    pub category: String,
    pub actual: i64,
    pub baseline: Option<i32>,
    pub variance: Option<i64>,
    pub variance_pct: Option<f64>,
}

fn actuals_by_category(
    workspace: &WorkspaceRef,
    start_date: &str,
    end_date: &str,
    conn: &mut SqliteConnection,
) -> Result<Vec<(CategoryRef, i64)>, AppError> {
    line_items::table
        .inner_join(transactions::table)
        .filter(transactions::workspace_id.eq(workspace))
        .filter(transactions::date.ge(start_date))
        .filter(transactions::date.le(end_date))
        .filter(transactions::cleared.eq(true))
        .filter(transactions::deleted_at.is_null())
        .group_by(line_items::category_id)
        .select((
            line_items::category_id,
            diesel::dsl::sum(line_items::amount).assume_not_null(),
        ))
        .load::<(CategoryRef, i64)>(conn)
        .map_err(|err| AppError::Internal(format!("actuals query failed: {err}")))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{Account, AccountType, NewAccount};
    use crate::domain::category::{Category, NewCategory};
    use crate::domain::transaction::{LineItemInput, NewTransaction};
    use crate::domain::user::NewUser;
    use crate::domain::workspace::{NewWorkspace, Workspace};
    use crate::test_helpers::setup_test_db;

    struct TestFixture {
        conn: SqliteConnection,
        workspace: Workspace,
        groceries: Category,
        dining: Category,
        chequing: Account,
    }

    fn setup() -> TestFixture {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace = NewWorkspace::new("My Finances", user.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        let groceries = NewCategory::new(workspace.as_ref().clone(), "Groceries")
            .insert(&mut conn)
            .unwrap();
        let dining = NewCategory::new(workspace.as_ref().clone(), "Dining")
            .insert(&mut conn)
            .unwrap();
        let chequing = NewAccount::new(workspace.as_ref().clone(), "Chequing", AccountType::Chequing)
            .insert(&mut conn)
            .unwrap();
        TestFixture {
            conn,
            workspace,
            groceries,
            dining,
            chequing,
        }
    }

    #[test]
    fn create_baseline_succeeds() {
        let mut fixture = setup();
        let baseline = NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            50000,
        )
        .insert(&mut fixture.conn)
        .expect("baseline creation should succeed");

        assert_eq!(baseline.amount, 50000);
        assert_eq!(baseline.period_type, "monthly");
        assert_eq!(baseline.period_start, "2026-04-01");
    }

    #[test]
    fn duplicate_baseline_rejected() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        let err = NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            60000,
        )
        .insert(&mut fixture.conn)
        .expect_err("duplicate should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_category_different_periods_allowed() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-05-01",
            55000,
        )
        .insert(&mut fixture.conn)
        .unwrap();
    }

    #[test]
    fn update_amount() {
        let mut fixture = setup();
        let baseline = NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        baseline.update_amount(60000, &mut fixture.conn).unwrap();

        let loaded = fixture
            .workspace
            .as_ref()
            .baselines("2026-04-01", &mut fixture.conn)
            .unwrap();
        assert_eq!(loaded[0].amount, 60000);
    }

    #[test]
    fn zero_baseline_allowed() {
        let mut fixture = setup();
        let baseline = NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            0,
        )
        .insert(&mut fixture.conn)
        .unwrap();
        assert_eq!(baseline.amount, 0);
    }

    #[test]
    fn variance_with_actuals_and_baseline() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            -50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            -45000,
            vec![LineItemInput::new(fixture.groceries.as_ref().clone(), -45000)],
        )
        .cleared(true)
        .insert(&mut fixture.conn)
        .unwrap();

        let summary = fixture
            .workspace
            .as_ref()
            .period_summary("2026-04-01", "2026-04-30", &mut fixture.conn)
            .unwrap();
        let groceries_variance = summary
            .iter()
            .find(|v| v.category == fixture.groceries.id.to_string())
            .unwrap();

        assert_eq!(groceries_variance.actual, -45000);
        assert_eq!(groceries_variance.baseline, Some(-50000));
        assert_eq!(groceries_variance.variance, Some(5000));
    }

    #[test]
    fn no_baseline_returns_none_variance() {
        let mut fixture = setup();
        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            -3000,
            vec![LineItemInput::new(fixture.dining.as_ref().clone(), -3000)],
        )
        .cleared(true)
        .insert(&mut fixture.conn)
        .unwrap();

        let summary = fixture
            .workspace
            .as_ref()
            .period_summary("2026-04-01", "2026-04-30", &mut fixture.conn)
            .unwrap();
        let dining_variance = summary
            .iter()
            .find(|v| v.category == fixture.dining.id.to_string())
            .unwrap();

        assert_eq!(dining_variance.actual, -3000);
        assert!(dining_variance.baseline.is_none());
        assert!(dining_variance.variance.is_none());
    }

    #[test]
    fn baseline_with_no_actuals_shows_full_variance() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            -50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        let summary = fixture
            .workspace
            .as_ref()
            .period_summary("2026-04-01", "2026-04-30", &mut fixture.conn)
            .unwrap();
        let groceries_variance = summary
            .iter()
            .find(|v| v.category == fixture.groceries.id.to_string())
            .unwrap();

        assert_eq!(groceries_variance.actual, 0);
        assert_eq!(groceries_variance.baseline, Some(-50000));
        assert_eq!(groceries_variance.variance, Some(50000));
    }

    #[test]
    fn pending_transactions_excluded_from_variance() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            -50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            -45000,
            vec![LineItemInput::new(fixture.groceries.as_ref().clone(), -45000)],
        )
        .insert(&mut fixture.conn)
        .unwrap();

        let summary = fixture
            .workspace
            .as_ref()
            .period_summary("2026-04-01", "2026-04-30", &mut fixture.conn)
            .unwrap();
        let groceries_variance = summary
            .iter()
            .find(|v| v.category == fixture.groceries.id.to_string())
            .unwrap();

        assert_eq!(groceries_variance.actual, 0);
    }

    #[test]
    fn period_summary_includes_all_active_categories() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            -50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            -3000,
            vec![LineItemInput::new(fixture.dining.as_ref().clone(), -3000)],
        )
        .cleared(true)
        .insert(&mut fixture.conn)
        .unwrap();

        let summary = fixture
            .workspace
            .as_ref()
            .period_summary("2026-04-01", "2026-04-30", &mut fixture.conn)
            .unwrap();
        assert_eq!(summary.len(), 2);

        let has_groceries = summary
            .iter()
            .any(|v| v.category == fixture.groceries.id.to_string());
        let has_dining = summary
            .iter()
            .any(|v| v.category == fixture.dining.id.to_string());
        assert!(has_groceries);
        assert!(has_dining);
    }

    #[test]
    fn variance_percentage() {
        let mut fixture = setup();
        NewBaseline::new(
            &fixture.workspace.as_ref(),
            &fixture.groceries.as_ref(),
            PeriodType::Monthly,
            "2026-04-01",
            -50000,
        )
        .insert(&mut fixture.conn)
        .unwrap();

        NewTransaction::new(
            &fixture.workspace.as_ref(),
            &fixture.chequing.as_ref(),
            "2026-04-10",
            -60000,
            vec![LineItemInput::new(fixture.groceries.as_ref().clone(), -60000)],
        )
        .cleared(true)
        .insert(&mut fixture.conn)
        .unwrap();

        let summary = fixture
            .workspace
            .as_ref()
            .period_summary("2026-04-01", "2026-04-30", &mut fixture.conn)
            .unwrap();
        let groceries_variance = summary
            .iter()
            .find(|v| v.category == fixture.groceries.id.to_string())
            .unwrap();

        assert_eq!(groceries_variance.variance, Some(-10000));
        let pct = groceries_variance.variance_pct.unwrap();
        assert!((pct - 20.0).abs() < 0.01);
    }
}
