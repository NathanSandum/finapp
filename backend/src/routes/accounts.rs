use axum::{extract::Extension, extract::Path, http::StatusCode, Json};
use diesel::SqliteConnection;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::{Account, AccountRef, AccountType, NewCustomAccount};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::jsend::{JSend, JSendResponse};
use crate::DbPool;

pub async fn list(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let accounts = member.workspace.accounts(&mut conn)?;

    let items: Vec<Value> = accounts
        .iter()
        .map(|account| account_to_json(account, &mut conn))
        .collect::<Result<_, _>>()?;

    Ok(JSend::success(json!({ "accounts": items })))
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    name: String,
    parent_id: Option<AccountRef>,
}

pub async fn create(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateAccountRequest>,
) -> Result<JSendResponse<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let parent_id = body.parent_id.as_ref().ok_or_else(|| {
        AppError::Validation(vec![crate::error::FieldError {
            field: "parent_id".into(),
            message: "parent_id is required; system accounts are seeded and cannot be created"
                .into(),
        }])
    })?;

    let parent = Account::from_ref(parent_id, &mut conn)?
        .ok_or(AppError::NotFound("parent account".into()))?;

    if parent.workspace != member.workspace {
        return Err(AppError::NotFound("parent account".into()));
    }

    let account = NewCustomAccount::child(&parent, &body.name).insert(&mut conn)?;

    let body = account_to_json(&account, &mut conn)?;
    Ok(JSendResponse::new(
        StatusCode::CREATED,
        JSend::success(body),
    ))
}

#[derive(Deserialize)]
pub struct AccountPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    account: AccountRef,
}

pub async fn delete(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountPath>,
) -> Result<JSend<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let account =
        Account::from_ref(&path.account, &mut conn)?.ok_or(AppError::NotFound("account".into()))?;

    if account.workspace != member.workspace {
        return Err(AppError::NotFound("account".into()));
    }

    account.soft_delete(&mut conn)?;

    Ok(JSend::success(Value::Null))
}

#[derive(Deserialize)]
pub struct UpdateAccountRequest {
    name: Option<String>,
    parent_id: Option<AccountRef>,
}

pub async fn update(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountPath>,
    Json(body): Json<UpdateAccountRequest>,
) -> Result<JSend<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let mut account =
        Account::from_ref(&path.account, &mut conn)?.ok_or(AppError::NotFound("account".into()))?;

    if account.workspace != member.workspace {
        return Err(AppError::NotFound("account".into()));
    }

    if let Some(ref new_name) = body.name {
        account = account.rename(new_name, &mut conn)?;
    }

    if let Some(ref new_parent_id) = body.parent_id {
        account = account.reparent(new_parent_id, &mut conn)?;
    }

    let body = account_to_json(&account, &mut conn)?;
    Ok(JSend::success(body))
}

pub async fn balance(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountPath>,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let account =
        Account::from_ref(&path.account, &mut conn)?.ok_or(AppError::NotFound("account".into()))?;

    if account.workspace != member.workspace {
        return Err(AppError::NotFound("account".into()));
    }

    let balance = account.balance(&mut conn)?;

    Ok(JSend::success(json!({
        "account": account.id,
        "balance": balance,
    })))
}

fn account_to_json(
    account: &Account,
    conn: &mut SqliteConnection,
) -> Result<Value, AppError> {
    let (kind, name) = match &account.account_type {
        AccountType::System(s) => (s.as_str().to_string(), Value::Null),
        AccountType::Custom(c) => ("Custom".to_string(), Value::String(c.name.clone())),
    };
    Ok(json!({
        "id": account.id,
        "parent_id": account.parent_id,
        "kind": kind,
        "name": name,
        "category": account.category(conn)?,
        "currency": account.currency,
    }))
}
