use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::{Account, AccountRef, AccountType, NewAccount};
use crate::domain::transaction::Transaction;
use crate::domain::workspace::Role;
use crate::error::AppError;
use crate::DbPool;

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let accounts = Account::for_workspace(&member.workspace, &mut conn)?;

    let items: Vec<Value> = accounts
        .iter()
        .map(|a| {
            json!({
                "id": a.id,
                "name": a.name,
                "account_type": a.account_type,
                "currency": a.currency,
            })
        })
        .collect();

    Ok(Json(json!({ "accounts": items })))
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    name: String,
    account_type: String,
    currency: Option<String>,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateAccountRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let account_type = AccountType::from_str(&body.account_type)
        .ok_or_else(|| AppError::Validation(vec![crate::error::FieldError {
            field: "account_type".into(),
            message: format!("unknown account type '{}'", body.account_type),
        }]))?;

    let mut builder = NewAccount::new(&member.workspace, &body.name, account_type);
    if let Some(ref currency) = body.currency {
        builder = builder.currency(currency);
    }
    let acct = builder.insert(&mut conn)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({
            "id": acct.id,
            "name": acct.name,
            "account_type": acct.account_type,
            "currency": acct.currency,
        })),
    ))
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path((_workspace_id, account_id)): Path<(String, AccountRef)>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let acct = Account::from_ref(&account_id, &mut conn)?
        .ok_or(AppError::NotFound("account".into()))?;

    if acct.workspace_id != member.workspace.id {
        return Err(AppError::NotFound("account".into()));
    }

    acct.soft_delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}

pub async fn balance(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path((_workspace_id, account_id)): Path<(String, AccountRef)>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let acct = Account::from_ref(&account_id, &mut conn)?
        .ok_or(AppError::NotFound("account".into()))?;

    if acct.workspace_id != member.workspace.id {
        return Err(AppError::NotFound("account".into()));
    }

    let cleared = Transaction::cleared_balance_for_account(&acct, &mut conn)?;

    Ok(Json(json!({
        "account_id": acct.id,
        "cleared_balance": cleared,
    })))
}
