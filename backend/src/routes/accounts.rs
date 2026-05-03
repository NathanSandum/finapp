use axum::{extract::Extension, extract::Path, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::{Account, AccountRef, AccountType, NewAccount};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::jsend::{JSend, JSendResponse};
use crate::DbPool;

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<JSend<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let accounts = member.workspace.accounts(&mut conn)?;

    let items: Vec<Value> = accounts.iter().map(account_to_json).collect();

    Ok(JSend::success(json!({ "accounts": items })))
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    name: String,
    parent_id: Option<AccountRef>,
    account_type: Option<String>,
    currency: Option<String>,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateAccountRequest>,
) -> Result<JSendResponse<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let mut builder = if let Some(ref parent_id) = body.parent_id {
        let parent = Account::from_ref(parent_id, &mut conn)?
            .ok_or(AppError::NotFound("parent account".into()))?;

        if parent.workspace != member.workspace {
            return Err(AppError::NotFound("parent account".into()));
        }

        NewAccount::child(&parent, &body.name)
    } else {
        let account_type_str = body.account_type.as_deref().ok_or_else(|| {
            AppError::Validation(vec![crate::error::FieldError {
                field: "account_type".into(),
                message: "account_type is required for root accounts".into(),
            }])
        })?;

        let account_type = AccountType::from_str(account_type_str).ok_or_else(|| {
            AppError::Validation(vec![crate::error::FieldError {
                field: "account_type".into(),
                message: format!("unknown account type '{account_type_str}'"),
            }])
        })?;

        NewAccount::root(member.workspace.clone(), &body.name, account_type)
    };

    if let Some(ref currency) = body.currency {
        builder = builder.currency(currency);
    }

    let account = builder.insert(&mut conn)?;

    Ok(JSendResponse::new(
        StatusCode::CREATED,
        JSend::success(account_to_json(&account)),
    ))
}

#[derive(Deserialize)]
pub struct AccountPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    account: AccountRef,
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountPath>,
) -> Result<JSend<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

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
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountPath>,
    Json(body): Json<UpdateAccountRequest>,
) -> Result<JSend<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

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

    Ok(JSend::success(account_to_json(&account)))
}

pub async fn balance(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountPath>,
) -> Result<JSend<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

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

fn account_to_json(account: &Account) -> Value {
    json!({
        "id": account.id,
        "parent_id": account.parent_id,
        "name": account.name,
        "account_type": account.account_type,
        "currency": account.currency,
    })
}
