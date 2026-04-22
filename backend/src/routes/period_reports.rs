use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::period::Period;
use crate::domain::report::{BalanceSheet, IncomeStatement};
use crate::domain::workspace::WorkspaceRef;
use crate::error::AppError;
use crate::DbPool;

#[derive(Deserialize)]
pub struct PeriodPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    year: String,
    period: String,
}

pub async fn income_statement(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<PeriodPath>,
) -> Result<Json<Value>, AppError> {
    let period = Period::parse(&path.year, &path.period)?;

    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let statement = IncomeStatement::for_period(
        &member.workspace,
        &period.start_date,
        &period.end_date,
        &mut conn,
    )?;

    Ok(Json(serde_json::to_value(statement).map_err(|err| {
        AppError::Internal(format!("serialization failed: {err}"))
    })?))
}

pub async fn balance_sheet(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<PeriodPath>,
) -> Result<Json<Value>, AppError> {
    let period = Period::parse(&path.year, &path.period)?;

    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let sheet = BalanceSheet::as_of(&member.workspace, &period.end_date, &mut conn)?;

    Ok(Json(serde_json::to_value(sheet).map_err(|err| {
        AppError::Internal(format!("serialization failed: {err}"))
    })?))
}

pub async fn entries(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<PeriodPath>,
) -> Result<Json<Value>, AppError> {
    let period = Period::parse(&path.year, &path.period)?;

    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entries = member
        .workspace
        .entries_between(&period.start_date, &period.end_date, &mut conn)?;

    let mut items = Vec::with_capacity(entries.len());
    for entry in &entries {
        let postings = entry.postings(&mut conn)?;
        items.push(super::entries::entry_to_json(entry, &postings));
    }

    Ok(Json(json!({ "entries": items })))
}
