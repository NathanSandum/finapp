use axum::{extract::Extension, extract::Path, extract::Query, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::baseline::{Baseline, NewBaseline, PeriodType};
use crate::domain::category::CategoryRef;
use crate::domain::workspace::Role;
use crate::error::AppError;
use crate::schema::{baselines, categories};
use crate::DbPool;
use diesel::prelude::*;

#[derive(Deserialize)]
pub struct ListQuery {
    period_start: String,
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let baselines =
        Baseline::for_workspace_period(&member.workspace, &query.period_start, &mut conn)?;

    let items: Vec<Value> = baselines
        .iter()
        .map(|b| {
            json!({
                "id": b.id,
                "category": b.category,
                "period_type": b.period_type,
                "period_start": b.period_start,
                "amount": b.amount,
            })
        })
        .collect();

    Ok(Json(json!({ "baselines": items })))
}

#[derive(Deserialize)]
pub struct CreateBaselineRequest {
    category: CategoryRef,
    period_type: String,
    period_start: String,
    amount: i32,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateBaselineRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let period_type = PeriodType::from_str(&body.period_type).ok_or_else(|| {
        AppError::Validation(vec![crate::error::FieldError {
            field: "period_type".into(),
            message: format!(
                "unknown period type '{}', expected 'monthly' or 'quarterly'",
                body.period_type
            ),
        }])
    })?;

    let category = categories::table
        .find(&body.category)
        .select(crate::domain::category::Category::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|err| AppError::Internal(format!("category lookup failed: {err}")))?
        .ok_or(AppError::NotFound("category".into()))?;

    if category.workspace != member.workspace {
        return Err(AppError::NotFound("category".into()));
    }

    let baseline = NewBaseline::new(
        &member.workspace,
        &body.category,
        period_type,
        &body.period_start,
        body.amount,
    )
    .insert(&mut conn)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": baseline.id,
            "category": baseline.category,
            "period_type": baseline.period_type,
            "period_start": baseline.period_start,
            "amount": baseline.amount,
        })),
    ))
}

#[derive(Deserialize)]
pub struct BaselinePath {
    #[allow(dead_code)]
    workspace: String,
    baseline: String,
}

#[derive(Deserialize)]
pub struct UpdateBaselineRequest {
    amount: i32,
}

pub async fn update(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<BaselinePath>,
    Json(body): Json<UpdateBaselineRequest>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let baseline = baselines::table
        .find(&path.baseline)
        .select(Baseline::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|err| AppError::Internal(format!("baseline lookup failed: {err}")))?
        .ok_or(AppError::NotFound("baseline".into()))?;

    if baseline.workspace != member.workspace {
        return Err(AppError::NotFound("baseline".into()));
    }

    baseline.update_amount(body.amount, &mut conn)?;

    Ok(Json(json!({
        "id": baseline.id,
        "category": baseline.category,
        "period_type": baseline.period_type,
        "period_start": baseline.period_start,
        "amount": body.amount,
    })))
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    period_start: String,
    period_end: String,
}

pub async fn summary(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Query(query): Query<SummaryQuery>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let variances =
        member
            .workspace
            .period_summary(&query.period_start, &query.period_end, &mut conn)?;

    let items: Vec<Value> = variances
        .iter()
        .map(|v| {
            json!({
                "category": v.category,
                "actual": v.actual,
                "baseline": v.baseline,
                "variance": v.variance,
                "variance_pct": v.variance_pct,
            })
        })
        .collect();

    Ok(Json(json!({ "summary": items })))
}
