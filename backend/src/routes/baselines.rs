use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::AccountRef;
use crate::domain::baseline::{Baseline, BaselineRef, NewBaseline};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::DbPool;

#[derive(Deserialize)]
pub struct CreateBaselineRequest {
    account_id: AccountRef,
    period_start: String,
    amount: i32,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateBaselineRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let baseline = NewBaseline::monthly(
        member.workspace.clone(),
        body.account_id,
        &body.period_start,
        body.amount,
    )
    .insert(&mut conn)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::to_value(&baseline).map_err(|err| {
            AppError::Internal(format!("serialization failed: {err}"))
        })?),
    ))
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let baselines = member.workspace.baselines(&mut conn)?;

    Ok(Json(json!({
        "baselines": serde_json::to_value(&baselines).map_err(|err| {
            AppError::Internal(format!("serialization failed: {err}"))
        })?
    })))
}

#[derive(Deserialize)]
pub struct BaselinePath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    baseline: BaselineRef,
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<BaselinePath>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let baseline = Baseline::from_ref(&path.baseline, &mut conn)?
        .ok_or(AppError::NotFound("baseline".into()))?;

    if baseline.workspace != member.workspace {
        return Err(AppError::NotFound("baseline".into()));
    }

    baseline.delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}

pub async fn variance(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<BaselinePath>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let baseline = Baseline::from_ref(&path.baseline, &mut conn)?
        .ok_or(AppError::NotFound("baseline".into()))?;

    if baseline.workspace != member.workspace {
        return Err(AppError::NotFound("baseline".into()));
    }

    let variance = baseline.variance(&mut conn)?;

    Ok(Json(json!({
        "baseline": baseline.id,
        "amount": baseline.amount,
        "actual": variance + baseline.amount as i64,
        "variance": variance,
    })))
}
