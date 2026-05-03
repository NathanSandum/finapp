use axum::{extract::Extension, extract::Query};
use serde::Deserialize;

use crate::auth::WorkspaceMember;
use crate::domain::report::{BalanceSheet, IncomeStatement};
use crate::error::{AppError, FieldError};
use crate::jsend::JSend;
use crate::DbPool;

#[derive(Deserialize)]
pub struct BalanceSheetQuery {
    as_of: Option<String>,
}

pub async fn balance_sheet(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Query(query): Query<BalanceSheetQuery>,
) -> Result<JSend<BalanceSheet>, AppError> {
    let as_of = query.as_of.ok_or_else(|| {
        AppError::Validation(vec![FieldError {
            field: "as_of".into(),
            message: "as_of query parameter is required".into(),
        }])
    })?;

    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let sheet = BalanceSheet::as_of(&member.workspace, &as_of, &mut conn)?;

    Ok(JSend::success(sheet))
}

#[derive(Deserialize)]
pub struct IncomeStatementQuery {
    start_date: Option<String>,
    end_date: Option<String>,
}

pub async fn income_statement(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Query(query): Query<IncomeStatementQuery>,
) -> Result<JSend<IncomeStatement>, AppError> {
    let mut errors = Vec::new();
    if query.start_date.is_none() {
        errors.push(FieldError {
            field: "start_date".into(),
            message: "start_date query parameter is required".into(),
        });
    }
    if query.end_date.is_none() {
        errors.push(FieldError {
            field: "end_date".into(),
            message: "end_date query parameter is required".into(),
        });
    }
    if !errors.is_empty() {
        return Err(AppError::Validation(errors));
    }

    let start_date = query.start_date.unwrap();
    let end_date = query.end_date.unwrap();

    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let statement = IncomeStatement::for_period(
        &member.workspace,
        &start_date,
        &end_date,
        &mut conn,
    )?;

    Ok(JSend::success(statement))
}
