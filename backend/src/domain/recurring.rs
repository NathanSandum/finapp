use chrono::{NaiveDate, NaiveDateTime, TimeZone};
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use rrule::{RRule, RRuleSet, Tz, Unvalidated};
use uuid::Uuid;

use crate::domain::account::AccountRef;
use crate::domain::workspace::WorkspaceRef;
use crate::error::{AppError, FieldError};
use crate::schema::{recurring_posting_templates, recurring_rules};

// ── Refs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct RecurringRuleRef(String);

impl std::fmt::Display for RecurringRuleRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct RecurringPostingTemplateRef(String);

impl std::fmt::Display for RecurringPostingTemplateRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Schedule ────────────────────────────────────────────────────────────────

/// A recurring schedule: an RRULE string anchored at a dtstart.
/// Existence of a `Schedule` value is proof that both have been parsed and validated.
#[derive(Debug, Clone)]
pub struct Schedule {
    rrule: String,
    dtstart: NaiveDate,
}

impl Schedule {
    /// Parse and validate an RRULE string against the given dtstart.
    /// The only constructor — once you hold a `Schedule`, it is well-formed.
    pub fn parse(rrule: &str, dtstart: NaiveDate) -> Result<Schedule, AppError> {
        let parsed: RRule<Unvalidated> = rrule.parse().map_err(|err| {
            AppError::Validation(vec![FieldError {
                field: "rrule".into(),
                message: format!("invalid RRULE: {err}"),
            }])
        })?;
        parsed.validate(date_at_utc(dtstart)).map_err(|err| {
            AppError::Validation(vec![FieldError {
                field: "rrule".into(),
                message: format!("invalid RRULE: {err}"),
            }])
        })?;
        Ok(Schedule {
            rrule: rrule.to_string(),
            dtstart,
        })
    }

    pub fn rrule(&self) -> &str {
        &self.rrule
    }

    pub fn dtstart(&self) -> NaiveDate {
        self.dtstart
    }

    /// First scheduled date strictly after `after_exclusive`.
    /// Returns None when the schedule is exhausted (past UNTIL / COUNT consumed)
    /// or when shifting past `after_exclusive` would overflow `NaiveDate`.
    pub fn next_after(&self, after_exclusive: NaiveDate) -> Option<NaiveDate> {
        // `RRuleSet::after` is inclusive — bump by one day so callers get strictly-after.
        let lower_bound = after_exclusive.succ_opt()?;
        let dtstart_dt = date_at_utc(self.dtstart);
        let parsed: RRule<Unvalidated> = self
            .rrule
            .parse()
            .expect("Schedule guarantees parseable RRULE");
        let validated = parsed
            .validate(dtstart_dt)
            .expect("Schedule guarantees validated RRULE");
        RRuleSet::new(dtstart_dt)
            .rrule(validated)
            .after(date_at_utc(lower_bound))
            .all(1)
            .dates
            .first()
            .map(|d| d.date_naive())
    }
}

fn date_at_utc(date: NaiveDate) -> chrono::DateTime<Tz> {
    let naive = date.and_hms_opt(0, 0, 0).expect("00:00:00 valid for any date");
    Tz::UTC.from_utc_datetime(&naive)
}

// ── PostingTemplateInput / PostingTemplates ─────────────────────────────────

#[derive(Debug)]
pub struct PostingTemplateInput {
    pub account: AccountRef,
    pub amount: i32,
    pub note: Option<String>,
}

impl PostingTemplateInput {
    pub fn new(account: AccountRef, amount: i32) -> Self {
        PostingTemplateInput {
            account,
            amount,
            note: None,
        }
    }

    pub fn note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
}

/// A set of posting templates that satisfies the double-entry invariant.
/// Existence of a `PostingTemplates` value is proof of: ≥2 entries, non-zero amounts, zero-sum.
#[derive(Debug)]
pub struct PostingTemplates(Vec<PostingTemplateInput>);

impl PostingTemplates {
    pub fn new(inputs: Vec<PostingTemplateInput>) -> Result<Self, AppError> {
        if inputs.len() < 2 {
            return Err(AppError::Validation(vec![FieldError {
                field: "templates".into(),
                message: "rule must have at least 2 posting templates".into(),
            }]));
        }
        for template in &inputs {
            if template.amount == 0 {
                return Err(AppError::Validation(vec![FieldError {
                    field: "templates".into(),
                    message: "template amount cannot be zero".into(),
                }]));
            }
        }
        let sum: i64 = inputs.iter().map(|t| t.amount as i64).sum();
        if sum != 0 {
            return Err(AppError::Validation(vec![FieldError {
                field: "templates".into(),
                message: format!("templates must sum to zero, got {sum}"),
            }]));
        }
        Ok(PostingTemplates(inputs))
    }

    pub fn into_inner(self) -> Vec<PostingTemplateInput> {
        self.0
    }
}

// ── RecurringRule ───────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, Queryable, Selectable)]
#[diesel(table_name = recurring_rules)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RecurringRule {
    pub id: RecurringRuleRef,
    #[diesel(column_name = workspace_id)]
    pub workspace: WorkspaceRef,
    pub description: String,
    pub rrule: String,
    pub dtstart: NaiveDate,
    pub last_fired_on: Option<NaiveDate>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl RecurringRule {
    pub fn from_ref(
        rule_ref: &RecurringRuleRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<RecurringRule>, AppError> {
        recurring_rules::table
            .find(rule_ref)
            .filter(recurring_rules::deleted_at.is_null())
            .select(RecurringRule::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("recurring rule lookup failed: {err}")))
    }

    pub fn templates(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<RecurringPostingTemplate>, AppError> {
        recurring_posting_templates::table
            .filter(recurring_posting_templates::rule_id.eq(&self.id))
            .select(RecurringPostingTemplate::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("templates query failed: {err}")))
    }

    pub fn soft_delete(&self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(recurring_rules::table.find(&self.id))
            .set(recurring_rules::deleted_at.eq(Some(now)))
            .execute(conn)
            .map_err(|err| AppError::Internal(format!("recurring rule soft delete failed: {err}")))?;
        Ok(())
    }

    /// Rebuild the `Schedule` from stored columns. Errors only if storage is corrupt.
    pub fn schedule(&self) -> Result<Schedule, AppError> {
        Schedule::parse(&self.rrule, self.dtstart)
            .map_err(|err| AppError::Internal(format!("stored schedule invalid: {err:?}")))
    }

    /// Next scheduled occurrence strictly after `after_exclusive`.
    pub fn next_occurrence_after(
        &self,
        after_exclusive: NaiveDate,
    ) -> Result<Option<NaiveDate>, AppError> {
        Ok(self.schedule()?.next_after(after_exclusive))
    }

    /// Next occurrence that has not yet been materialized.
    /// Uses `last_fired_on` as exclusive lower bound; if never fired, uses the day before
    /// `dtstart` so the first call returns the `dtstart` occurrence itself
    /// (assuming the RRULE permits a hit on that day).
    pub fn next_pending(&self) -> Result<Option<NaiveDate>, AppError> {
        let cursor = match self.last_fired_on {
            Some(date) => date,
            None => self
                .dtstart
                .pred_opt()
                .ok_or_else(|| AppError::Internal("dtstart underflow".into()))?,
        };
        self.next_occurrence_after(cursor)
    }
}

// ── RecurringPostingTemplate ────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, Queryable, Selectable)]
#[diesel(table_name = recurring_posting_templates)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RecurringPostingTemplate {
    pub id: RecurringPostingTemplateRef,
    #[diesel(column_name = rule_id)]
    pub rule: RecurringRuleRef,
    #[diesel(column_name = account_id)]
    pub account: AccountRef,
    pub amount: i32,
    pub note: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ── Query methods on owning types ───────────────────────────────────────────

impl WorkspaceRef {
    pub fn recurring_rules(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<RecurringRule>, AppError> {
        recurring_rules::table
            .filter(recurring_rules::workspace_id.eq(self))
            .filter(recurring_rules::deleted_at.is_null())
            .select(RecurringRule::as_select())
            .order(recurring_rules::dtstart.asc())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("recurring rules query failed: {err}")))
    }
}

// ── NewRecurringRule ────────────────────────────────────────────────────────

pub struct NewRecurringRule {
    workspace: WorkspaceRef,
    description: String,
    schedule: Schedule,
    templates: PostingTemplates,
}

#[derive(Insertable)]
#[diesel(table_name = recurring_rules)]
struct NewRecurringRuleRow {
    id: String,
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    description: String,
    rrule: String,
    dtstart: NaiveDate,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = recurring_posting_templates)]
struct NewTemplateRow {
    id: String,
    #[diesel(column_name = rule_id)]
    rule: RecurringRuleRef,
    #[diesel(column_name = account_id)]
    account: AccountRef,
    amount: i32,
    note: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl NewRecurringRule {
    /// Takes only validated inputs — `Schedule` and `PostingTemplates` carry their invariants
    /// at the type level, so this constructor itself cannot produce an invalid rule.
    pub fn new(
        workspace: WorkspaceRef,
        description: &str,
        schedule: Schedule,
        templates: PostingTemplates,
    ) -> Self {
        NewRecurringRule {
            workspace,
            description: description.to_string(),
            schedule,
            templates,
        }
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<RecurringRule, AppError> {
        conn.transaction::<_, AppError, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();
            let rule = diesel::insert_into(recurring_rules::table)
                .values(NewRecurringRuleRow {
                    id: Uuid::new_v4().to_string(),
                    workspace: self.workspace,
                    description: self.description,
                    rrule: self.schedule.rrule().to_string(),
                    dtstart: self.schedule.dtstart(),
                    created_at: now,
                    updated_at: now,
                })
                .returning(RecurringRule::as_returning())
                .get_result(conn)?;

            let template_rows: Vec<NewTemplateRow> = self
                .templates
                .into_inner()
                .into_iter()
                .map(|template| NewTemplateRow {
                    id: Uuid::new_v4().to_string(),
                    rule: rule.id.clone(),
                    account: template.account,
                    amount: template.amount,
                    note: template.note.unwrap_or_default(),
                    created_at: now,
                    updated_at: now,
                })
                .collect();

            diesel::insert_into(recurring_posting_templates::table)
                .values(&template_rows)
                .execute(conn)?;

            Ok(rule)
        })
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::Account;
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

    fn nd(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn rent_templates(chequing: &Account, rent: &Account) -> PostingTemplates {
        PostingTemplates::new(vec![
            PostingTemplateInput::new(chequing.id.clone(), -150_000),
            PostingTemplateInput::new(rent.id.clone(), 150_000),
        ])
        .unwrap()
    }

    fn monthly_first(date: NaiveDate) -> Schedule {
        Schedule::parse("FREQ=MONTHLY;BYMONTHDAY=1", date).unwrap()
    }

    // ── Schedule construction ─────────────────────────────────────────────

    #[test]
    fn schedule_parse_succeeds_for_valid_rrule() {
        assert!(Schedule::parse("FREQ=MONTHLY;BYMONTHDAY=1", nd("2026-01-01")).is_ok());
        assert!(Schedule::parse("FREQ=WEEKLY;INTERVAL=2", nd("2026-01-05")).is_ok());
        assert!(
            Schedule::parse("FREQ=MONTHLY;BYMONTHDAY=1;COUNT=3", nd("2026-01-01")).is_ok()
        );
    }

    #[test]
    fn schedule_parse_rejects_invalid_rrule() {
        let err = Schedule::parse("not an rrule", nd("2026-01-01"))
            .expect_err("garbage rrule should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn schedule_parse_rejects_empty_rrule() {
        let err = Schedule::parse("", nd("2026-01-01")).expect_err("empty rrule should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ── PostingTemplates construction ─────────────────────────────────────

    #[test]
    fn posting_templates_rejects_fewer_than_two() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);

        let err = PostingTemplates::new(vec![PostingTemplateInput::new(chequing.id.clone(), 100)])
            .expect_err("single template should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn posting_templates_rejects_non_zero_sum() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let err = PostingTemplates::new(vec![
            PostingTemplateInput::new(chequing.id.clone(), -150_000),
            PostingTemplateInput::new(rent.id.clone(), 100_000),
        ])
        .expect_err("non-zero sum should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn posting_templates_rejects_zero_amount() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let err = PostingTemplates::new(vec![
            PostingTemplateInput::new(chequing.id.clone(), 0),
            PostingTemplateInput::new(rent.id.clone(), 0),
        ])
        .expect_err("zero amount should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ── Insert / read ─────────────────────────────────────────────────────

    #[test]
    fn create_recurring_rule_succeeds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let rule = NewRecurringRule::new(
            workspace.as_ref().clone(),
            "Monthly rent",
            monthly_first(nd("2026-01-01")),
            rent_templates(&chequing, &rent),
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(rule.description, "Monthly rent");
        assert_eq!(rule.dtstart, nd("2026-01-01"));
        assert!(rule.last_fired_on.is_none());
        assert!(rule.deleted_at.is_none());

        let templates = rule.templates(&mut conn).unwrap();
        assert_eq!(templates.len(), 2);
        let sum: i64 = templates.iter().map(|t| t.amount as i64).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn templates_with_notes_persisted() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let templates = PostingTemplates::new(vec![
            PostingTemplateInput::new(chequing.id.clone(), -150_000).note("autopay"),
            PostingTemplateInput::new(rent.id.clone(), 150_000),
        ])
        .unwrap();

        let rule = NewRecurringRule::new(
            workspace.as_ref().clone(),
            "Rent",
            monthly_first(nd("2026-01-01")),
            templates,
        )
        .insert(&mut conn)
        .unwrap();

        let stored = rule.templates(&mut conn).unwrap();
        let chequing_template = stored.iter().find(|t| t.amount == -150_000).unwrap();
        assert_eq!(chequing_template.note, "autopay");
    }

    // ── Soft delete + list ────────────────────────────────────────────────

    #[test]
    fn soft_deleted_excluded_from_workspace_list() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let rule = NewRecurringRule::new(
            workspace.as_ref().clone(),
            "Rent",
            monthly_first(nd("2026-01-01")),
            rent_templates(&chequing, &rent),
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(workspace.as_ref().recurring_rules(&mut conn).unwrap().len(), 1);

        rule.soft_delete(&mut conn).unwrap();

        assert!(workspace.as_ref().recurring_rules(&mut conn).unwrap().is_empty());
        assert!(RecurringRule::from_ref(&rule.id, &mut conn).unwrap().is_none());
    }

    #[test]
    fn workspace_list_ordered_by_dtstart() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        NewRecurringRule::new(
            workspace.as_ref().clone(),
            "Rent",
            monthly_first(nd("2026-03-01")),
            rent_templates(&chequing, &rent),
        )
        .insert(&mut conn)
        .unwrap();

        let groceries_templates = PostingTemplates::new(vec![
            PostingTemplateInput::new(chequing.id.clone(), -5_000),
            PostingTemplateInput::new(food.id.clone(), 5_000),
        ])
        .unwrap();

        NewRecurringRule::new(
            workspace.as_ref().clone(),
            "Groceries",
            Schedule::parse("FREQ=WEEKLY", nd("2026-01-05")).unwrap(),
            groceries_templates,
        )
        .insert(&mut conn)
        .unwrap();

        let rules = workspace.as_ref().recurring_rules(&mut conn).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].description, "Groceries");
        assert_eq!(rules[1].description, "Rent");
    }

    // ── Schedule.next_after ───────────────────────────────────────────────

    #[test]
    fn next_after_monthly_first_of_month() {
        let schedule = monthly_first(nd("2026-01-01"));
        assert_eq!(schedule.next_after(nd("2026-01-15")), Some(nd("2026-02-01")));
        assert_eq!(schedule.next_after(nd("2026-01-31")), Some(nd("2026-02-01")));
        assert_eq!(schedule.next_after(nd("2026-02-01")), Some(nd("2026-03-01")));
    }

    #[test]
    fn next_after_weekly_interval_two() {
        let schedule = Schedule::parse("FREQ=WEEKLY;INTERVAL=2", nd("2026-01-05")).unwrap();
        assert_eq!(schedule.next_after(nd("2026-01-05")), Some(nd("2026-01-19")));
    }

    #[test]
    fn next_after_respects_until() {
        let schedule = Schedule::parse(
            "FREQ=MONTHLY;BYMONTHDAY=1;UNTIL=20260301T000000Z",
            nd("2026-01-01"),
        )
        .unwrap();
        assert_eq!(schedule.next_after(nd("2026-02-01")), Some(nd("2026-03-01")));
        assert_eq!(schedule.next_after(nd("2026-03-01")), None);
    }

    #[test]
    fn next_after_respects_count() {
        let schedule =
            Schedule::parse("FREQ=MONTHLY;BYMONTHDAY=1;COUNT=3", nd("2026-01-01")).unwrap();
        assert_eq!(schedule.next_after(nd("2026-03-01")), None);
    }

    #[test]
    fn next_after_bymonthday_last_day() {
        let schedule = Schedule::parse("FREQ=MONTHLY;BYMONTHDAY=-1", nd("2026-01-31")).unwrap();
        assert_eq!(schedule.next_after(nd("2026-01-31")), Some(nd("2026-02-28")));
    }

    // ── RecurringRule.next_pending ────────────────────────────────────────

    #[test]
    fn next_pending_never_fired_returns_dtstart() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        let rule = NewRecurringRule::new(
            workspace.as_ref().clone(),
            "Rent",
            monthly_first(nd("2026-01-01")),
            rent_templates(&chequing, &rent),
        )
        .insert(&mut conn)
        .unwrap();

        assert_eq!(rule.next_pending().unwrap(), Some(nd("2026-01-01")));
    }
}
