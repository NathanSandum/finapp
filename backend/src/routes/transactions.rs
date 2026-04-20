use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::{Account, AccountRef};
use crate::domain::transaction::{LineItemInput, NewTransaction, Transaction, TransactionRef};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::DbPool;

#[derive(Deserialize)]
pub struct AccountTransactionsPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    account: AccountRef,
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<AccountTransactionsPath>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let account =
        Account::from_ref(&path.account, &mut conn)?.ok_or(AppError::NotFound("account".into()))?;

    if account.workspace != member.workspace {
        return Err(AppError::NotFound("account".into()));
    }

    let transactions = path.account.transactions(&mut conn)?;

    let mut items = Vec::with_capacity(transactions.len());
    for transaction in &transactions {
        let line_items = transaction.line_items(&mut conn)?;
        let li_json: Vec<Value> = line_items
            .iter()
            .map(|li| {
                json!({
                    "id": li.id,
                    "category": li.category,
                    "amount": li.amount,
                    "note": li.note,
                })
            })
            .collect();

        items.push(json!({
            "id": transaction.id,
            "account": transaction.account,
            "transfer_account": transaction.transfer_account,
            "date": transaction.date,
            "description": transaction.description,
            "amount": transaction.amount,
            "cleared": transaction.cleared,
            "line_items": li_json,
        }));
    }

    Ok(Json(json!({ "transactions": items })))
}

#[derive(Deserialize)]
pub struct LineItemRequest {
    category_id: crate::domain::category::CategoryRef,
    amount: i32,
    note: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    account: AccountRef,
    transfer_account: Option<AccountRef>,
    date: String,
    description: Option<String>,
    amount: i32,
    cleared: Option<bool>,
    line_items: Vec<LineItemRequest>,
}

pub async fn create(
    member: WorkspaceMember,
    Extension(pool): Extension<DbPool>,
    Json(body): Json<CreateTransactionRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let account =
        Account::from_ref(&body.account, &mut conn)?.ok_or(AppError::NotFound("account".into()))?;

    if account.workspace != member.workspace {
        return Err(AppError::NotFound("account".into()));
    }

    let line_items: Vec<LineItemInput> = body
        .line_items
        .into_iter()
        .map(|li| {
            let mut input = LineItemInput::new(li.category_id.clone(), li.amount);
            if let Some(note) = li.note {
                input = input.note(&note);
            }
            input
        })
        .collect();

    let mut builder = NewTransaction::new(
        member.workspace.clone(),
        body.account.clone(),
        &body.date,
        body.amount,
        line_items,
    );

    if let Some(ref desc) = body.description {
        builder = builder.description(desc);
    }
    if let Some(cleared) = body.cleared {
        builder = builder.cleared(cleared);
    }
    if let Some(ref transfer_ref) = body.transfer_account {
        let transfer_account = Account::from_ref(transfer_ref, &mut conn)?
            .ok_or(AppError::NotFound("transfer account".into()))?;
        if transfer_account.workspace != member.workspace {
            return Err(AppError::NotFound("transfer account".into()));
        }
        builder = builder.transfer_to(&transfer_ref);
    }

    let transaction = builder.insert(&mut conn)?;

    let items = transaction.line_items(&mut conn)?;
    let li_json: Vec<Value> = items
        .iter()
        .map(|li| {
            json!({
                "id": li.id,
                "category": li.category,
                "amount": li.amount,
                "note": li.note,
            })
        })
        .collect();

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({
            "id": transaction.id,
            "account": transaction.account,
            "transfer_account": transaction.transfer_account,
            "date": transaction.date,
            "description": transaction.description,
            "amount": transaction.amount,
            "cleared": transaction.cleared,
            "line_items": li_json,
        })),
    ))
}

#[derive(Deserialize)]
pub struct DeleteTransactionPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    transaction: TransactionRef,
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<DeleteTransactionPath>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let transaction = Transaction::from_ref(&path.transaction, &mut conn)?
        .ok_or(AppError::NotFound("transaction".into()))?;

    if transaction.workspace != member.workspace {
        return Err(AppError::NotFound("transaction".into()));
    }

    transaction.soft_delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
