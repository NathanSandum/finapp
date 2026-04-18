use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use uuid::Uuid;

use crate::domain::category::Category;
use crate::domain::workspace::Workspace;
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
    pub workspace_id: String,
    pub category_id: String,
    pub period_type: String,
    pub period_start: String,
    pub amount: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Baseline {
    pub fn for_workspace_period(
        workspace: &Workspace,
        period_start: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Baseline>, AppError> {
        baselines::table
            .filter(baselines::workspace_id.eq(&workspace.id))
            .filter(baselines::period_start.eq(period_start))
            .select(Baseline::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("baseline list failed: {err}")))
    }

    pub fn update_amount(&self, amount: i32, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(baselines::table.find(&self.id))
            .set((
                baselines::amount.eq(amount),
                baselines::updated_at.eq(now),
            ))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("baseline update failed: {err}")))?;
        Ok(())
    }
}

// ── NewBaseline ──────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = baselines)]
struct NewBaselineRow {
    id: String,
    workspace_id: String,
    category_id: String,
    period_type: String,
    period_start: String,
    amount: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

pub struct NewBaseline {
    workspace_id: String,
    category_id: String,
    period_type: PeriodType,
    period_start: String,
    amount: i32,
}

impl NewBaseline {
    pub fn new(
        workspace: &Workspace,
        category: &Category,
        period_type: PeriodType,
        period_start: &str,
        amount: i32,
    ) -> Self {
        NewBaseline {
            workspace_id: workspace.id.clone(),
            category_id: category.id.clone(),
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
                workspace_id: self.workspace_id,
                category_id: self.category_id,
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
    pub category_id: String,
    pub actual: i64,
    pub baseline: Option<i32>,
    pub variance: Option<i64>,
    pub variance_pct: Option<f64>,
}

/// Compute actuals per category for a workspace in a date range.
fn actuals_by_category(
    workspace: &Workspace,
    start_date: &str,
    end_date: &str,
    conn: &mut SqliteConnection,
) -> Result<Vec<(String, i64)>, AppError> {
    // Join line_items → transactions, filter by workspace + date range + cleared + not deleted
    line_items::table
        .inner_join(transactions::table)
        .filter(transactions::workspace_id.eq(&workspace.id))
        .filter(transactions::date.ge(start_date))
        .filter(transactions::date.le(end_date))
        .filter(transactions::cleared.eq(true))
        .filter(transactions::deleted_at.is_null())
        .group_by(line_items::category_id)
        .select((
            line_items::category_id,
            diesel::dsl::sum(line_items::amount).assume_not_null(),
        ))
        .load::<(String, i64)>(conn)
        .map_err(|err| AppError::Internal(format!("actuals query failed: {err}")))
}

/// Compute period summary: for each category with activity or a baseline,
/// return actual, baseline, variance, and percentage.
pub fn period_summary(
    workspace: &Workspace,
    period_start: &str,
    period_end: &str,
    conn: &mut SqliteConnection,
) -> Result<Vec<CategoryVariance>, AppError> {
    let baselines = Baseline::for_workspace_period(workspace, period_start, conn)?;
    let actuals = actuals_by_category(workspace, period_start, period_end, conn)?;

    let mut results: Vec<CategoryVariance> = Vec::new();
    let mut seen_categories: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Categories with baselines
    for b in &baselines {
        let actual = actuals
            .iter()
            .find(|(cat_id, _)| cat_id == &b.category_id)
            .map(|(_, amt)| *amt)
            .unwrap_or(0);

        let variance = actual - b.amount as i64;
        let variance_pct = if b.amount != 0 {
            Some((variance as f64 / b.amount as f64) * 100.0)
        } else if actual != 0 {
            None // baseline is zero but actuals exist — percentage is undefined
        } else {
            Some(0.0) // both zero
        };

        results.push(CategoryVariance {
            category_id: b.category_id.clone(),
            actual,
            baseline: Some(b.amount),
            variance: Some(variance),
            variance_pct,
        });
        seen_categories.insert(b.category_id.clone());
    }

    // Categories with actuals but no baseline
    for (cat_id, actual) in &actuals {
        if !seen_categories.contains(cat_id) {
            results.push(CategoryVariance {
                category_id: cat_id.clone(),
                actual: *actual,
                baseline: None,
                variance: None,
                variance_pct: None,
            });
        }
    }

    Ok(results)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{AccountType, NewAccount};
    use crate::domain::category::NewCategory;
    use crate::domain::transaction::{LineItemInput, NewTransaction};
    use crate::domain::user::NewUser;
    use crate::domain::workspace::NewWorkspace;
    use crate::test_helpers::setup_test_db;

    struct TestFixture {
        conn: SqliteConnection,
        ws: Workspace,
        groceries: Category,
        dining: Category,
        chequing: crate::domain::account::Account,
    }

    fn setup() -> TestFixture {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap().insert(&mut conn).unwrap();
        let ws = NewWorkspace::new("My Finances", &user)
            .insert(&mut conn).unwrap();
        let groceries = NewCategory::new(&ws, "Groceries").insert(&mut conn).unwrap();
        let dining = NewCategory::new(&ws, "Dining").insert(&mut conn).unwrap();
        let chequing = NewAccount::new(&ws, "Chequing", AccountType::Chequing)
            .insert(&mut conn).unwrap();
        TestFixture { conn, ws, groceries, dining, chequing }
    }

    #[test]
    fn create_baseline_succeeds() {
        let mut f = setup();
        let bl = NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", 50000)
            .insert(&mut f.conn)
            .expect("baseline creation should succeed");

        assert_eq!(bl.amount, 50000);
        assert_eq!(bl.period_type, "monthly");
        assert_eq!(bl.period_start, "2026-04-01");
    }

    #[test]
    fn duplicate_baseline_rejected() {
        let mut f = setup();
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", 50000)
            .insert(&mut f.conn).unwrap();

        let err = NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", 60000)
            .insert(&mut f.conn)
            .expect_err("duplicate should fail");

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_category_different_periods_allowed() {
        let mut f = setup();
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", 50000)
            .insert(&mut f.conn).unwrap();
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-05-01", 55000)
            .insert(&mut f.conn).unwrap();
    }

    #[test]
    fn update_amount() {
        let mut f = setup();
        let bl = NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", 50000)
            .insert(&mut f.conn).unwrap();

        bl.update_amount(60000, &mut f.conn).unwrap();

        let loaded = Baseline::for_workspace_period(&f.ws, "2026-04-01", &mut f.conn).unwrap();
        assert_eq!(loaded[0].amount, 60000);
    }

    #[test]
    fn zero_baseline_allowed() {
        let mut f = setup();
        let bl = NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", 0)
            .insert(&mut f.conn).unwrap();
        assert_eq!(bl.amount, 0);
    }

    #[test]
    fn variance_with_actuals_and_baseline() {
        let mut f = setup();
        // Baseline: groceries $500
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", -50000)
            .insert(&mut f.conn).unwrap();

        // Actual: groceries $450 spent
        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", -45000,
            vec![LineItemInput::new(&f.groceries, -45000)],
        ).cleared(true).insert(&mut f.conn).unwrap();

        let summary = period_summary(&f.ws, "2026-04-01", "2026-04-30", &mut f.conn).unwrap();
        let groceries_var = summary.iter().find(|v| v.category_id == f.groceries.id).unwrap();

        assert_eq!(groceries_var.actual, -45000);
        assert_eq!(groceries_var.baseline, Some(-50000));
        assert_eq!(groceries_var.variance, Some(5000)); // spent less than baseline
    }

    #[test]
    fn no_baseline_returns_none_variance() {
        let mut f = setup();
        // Actual spending with no baseline set
        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", -3000,
            vec![LineItemInput::new(&f.dining, -3000)],
        ).cleared(true).insert(&mut f.conn).unwrap();

        let summary = period_summary(&f.ws, "2026-04-01", "2026-04-30", &mut f.conn).unwrap();
        let dining_var = summary.iter().find(|v| v.category_id == f.dining.id).unwrap();

        assert_eq!(dining_var.actual, -3000);
        assert!(dining_var.baseline.is_none());
        assert!(dining_var.variance.is_none());
    }

    #[test]
    fn baseline_with_no_actuals_shows_full_variance() {
        let mut f = setup();
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", -50000)
            .insert(&mut f.conn).unwrap();

        let summary = period_summary(&f.ws, "2026-04-01", "2026-04-30", &mut f.conn).unwrap();
        let groceries_var = summary.iter().find(|v| v.category_id == f.groceries.id).unwrap();

        assert_eq!(groceries_var.actual, 0);
        assert_eq!(groceries_var.baseline, Some(-50000));
        assert_eq!(groceries_var.variance, Some(50000)); // 0 - (-50000) = 50000
    }

    #[test]
    fn pending_transactions_excluded_from_variance() {
        let mut f = setup();
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", -50000)
            .insert(&mut f.conn).unwrap();

        // Pending transaction — should not count
        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", -45000,
            vec![LineItemInput::new(&f.groceries, -45000)],
        ).insert(&mut f.conn).unwrap();

        let summary = period_summary(&f.ws, "2026-04-01", "2026-04-30", &mut f.conn).unwrap();
        let groceries_var = summary.iter().find(|v| v.category_id == f.groceries.id).unwrap();

        assert_eq!(groceries_var.actual, 0); // pending excluded
    }

    #[test]
    fn period_summary_includes_all_active_categories() {
        let mut f = setup();
        // Baseline on groceries, actuals on dining, both should appear
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", -50000)
            .insert(&mut f.conn).unwrap();

        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", -3000,
            vec![LineItemInput::new(&f.dining, -3000)],
        ).cleared(true).insert(&mut f.conn).unwrap();

        let summary = period_summary(&f.ws, "2026-04-01", "2026-04-30", &mut f.conn).unwrap();
        assert_eq!(summary.len(), 2);

        let has_groceries = summary.iter().any(|v| v.category_id == f.groceries.id);
        let has_dining = summary.iter().any(|v| v.category_id == f.dining.id);
        assert!(has_groceries);
        assert!(has_dining);
    }

    #[test]
    fn variance_percentage() {
        let mut f = setup();
        NewBaseline::new(&f.ws, &f.groceries, PeriodType::Monthly, "2026-04-01", -50000)
            .insert(&mut f.conn).unwrap();

        NewTransaction::new(
            &f.ws, &f.chequing, "2026-04-10", -60000,
            vec![LineItemInput::new(&f.groceries, -60000)],
        ).cleared(true).insert(&mut f.conn).unwrap();

        let summary = period_summary(&f.ws, "2026-04-01", "2026-04-30", &mut f.conn).unwrap();
        let groceries_var = summary.iter().find(|v| v.category_id == f.groceries.id).unwrap();

        // actual -60000, baseline -50000, variance = -10000
        // pct = -10000 / -50000 * 100 = 20%
        assert_eq!(groceries_var.variance, Some(-10000));
        let pct = groceries_var.variance_pct.unwrap();
        assert!((pct - 20.0).abs() < 0.01);
    }
}
