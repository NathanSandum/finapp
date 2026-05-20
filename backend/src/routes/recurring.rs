use axum::{extract::Extension, extract::Path, http::StatusCode, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::AccountRef;
use crate::domain::recurring::{
    NewRecurringRule, PostingTemplateInput, PostingTemplates, RecurringPostingTemplate,
    RecurringRule, RecurringRuleRef, Schedule,
};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::jsend::{JSend, JSendResponse};
use crate::DbPool;

// ── Schedule preset enum (route-layer only) ─────────────────────────────────
//
// The wire format the frontend speaks is a tagged enum, not raw RRULE. This
// keeps the API surface small: the backend owns the allowlist, and changing
// the set of presets never breaks the storage shape (RRULE is the source of
// truth in the domain).
//
// Conversion lives entirely here. The domain knows nothing about presets.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SchedulePreset {
    Weekly,
    Biweekly,
    MonthlyFirst,
    MonthlyLast,
    Yearly,
}

impl SchedulePreset {
    fn rrule(self) -> &'static str {
        match self {
            SchedulePreset::Weekly => "FREQ=WEEKLY",
            SchedulePreset::Biweekly => "FREQ=WEEKLY;INTERVAL=2",
            SchedulePreset::MonthlyFirst => "FREQ=MONTHLY;BYMONTHDAY=1",
            SchedulePreset::MonthlyLast => "FREQ=MONTHLY;BYMONTHDAY=-1",
            SchedulePreset::Yearly => "FREQ=YEARLY",
        }
    }

    fn from_rrule(rrule: &str) -> Option<Self> {
        match rrule {
            "FREQ=WEEKLY" => Some(SchedulePreset::Weekly),
            "FREQ=WEEKLY;INTERVAL=2" => Some(SchedulePreset::Biweekly),
            "FREQ=MONTHLY;BYMONTHDAY=1" => Some(SchedulePreset::MonthlyFirst),
            "FREQ=MONTHLY;BYMONTHDAY=-1" => Some(SchedulePreset::MonthlyLast),
            "FREQ=YEARLY" => Some(SchedulePreset::Yearly),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
pub struct TemplateBody {
    account_id: AccountRef,
    amount: i32,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRecurringRequest {
    description: String,
    schedule: SchedulePreset,
    dtstart: NaiveDate,
    templates: Vec<TemplateBody>,
}

#[derive(Serialize)]
pub struct RecurringRuleResponse {
    #[serde(flatten)]
    rule: RecurringRule,
    /// Tagged enum echoed back so the frontend can render a label without
    /// parsing the stored RRULE. None if the stored RRULE doesn't match any
    /// preset (e.g. a rule created before the enum existed).
    schedule: Option<SchedulePreset>,
    templates: Vec<RecurringPostingTemplate>,
    next_pending: Option<NaiveDate>,
}

impl RecurringRuleResponse {
    fn build(
        rule: RecurringRule,
        conn: &mut diesel::SqliteConnection,
    ) -> Result<Self, AppError> {
        let templates = rule.templates(conn)?;
        let next_pending = rule.next_pending()?;
        let schedule = SchedulePreset::from_rrule(&rule.rrule);
        Ok(RecurringRuleResponse {
            rule,
            schedule,
            templates,
            next_pending,
        })
    }
}

pub async fn create(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateRecurringRequest>,
) -> Result<JSendResponse<RecurringRuleResponse>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let schedule = Schedule::parse(body.schedule.rrule(), body.dtstart)?;
    let template_inputs: Vec<PostingTemplateInput> = body
        .templates
        .into_iter()
        .map(|t| {
            let input = PostingTemplateInput::new(t.account_id, t.amount);
            match t.note {
                Some(note) => input.note(&note),
                None => input,
            }
        })
        .collect();
    let templates = PostingTemplates::new(template_inputs)?;

    let rule = NewRecurringRule::new(
        member.workspace.clone(),
        &body.description,
        schedule,
        templates,
    )
    .insert(&mut conn)?;

    let response = RecurringRuleResponse::build(rule, &mut conn)?;
    Ok(JSendResponse::new(StatusCode::CREATED, JSend::success(response)))
}

pub async fn list(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let rules = member.workspace.recurring_rules(&mut conn)?;
    let mut payload = Vec::with_capacity(rules.len());
    for rule in rules {
        payload.push(RecurringRuleResponse::build(rule, &mut conn)?);
    }

    Ok(JSend::success(json!({ "recurring_rules": payload })))
}

#[derive(Deserialize)]
pub struct RecurringPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    rule: RecurringRuleRef,
}

pub async fn get(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<RecurringPath>,
) -> Result<JSend<RecurringRuleResponse>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let rule = RecurringRule::from_ref(&path.rule, &mut conn)?
        .ok_or(AppError::NotFound("recurring rule".into()))?;

    if rule.workspace != member.workspace {
        return Err(AppError::NotFound("recurring rule".into()));
    }

    Ok(JSend::success(RecurringRuleResponse::build(rule, &mut conn)?))
}

pub async fn delete(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<RecurringPath>,
) -> Result<JSend<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let rule = RecurringRule::from_ref(&path.rule, &mut conn)?
        .ok_or(AppError::NotFound("recurring rule".into()))?;

    if rule.workspace != member.workspace {
        return Err(AppError::NotFound("recurring rule".into()));
    }

    rule.soft_delete(&mut conn)?;
    Ok(JSend::success(Value::Null))
}

/// Creates a "run": fires every due occurrence across all active rules in the
/// workspace as real entries. Idempotent — calling repeatedly with no time
/// passage posts zero new entries. Returns the run summary.
pub async fn create_run(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<JSendResponse<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let now = chrono::Utc::now().naive_utc();
    let today = now.date();
    let rules = member.workspace.recurring_rules(&mut conn)?;

    let mut posted_count: usize = 0;
    for rule in rules {
        posted_count += rule.run_due(today, &mut conn)?.len();
    }

    Ok(JSendResponse::new(
        StatusCode::CREATED,
        JSend::success(json!({ "posted": posted_count, "at": now })),
    ))
}
