use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::{Account, AccountRef};
use crate::domain::transaction::{LineItemInput, NewTransaction, Transaction, TransactionRef};
use crate::domain::workspace::Role;
use crate::error::AppError;
use crate::DbPool;

pub async fn list(
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

    let transactions = Transaction::for_account(&acct, &mut conn)?;

    let mut items = Vec::with_capacity(transactions.len());
    for txn in &transactions {
        let line_items = txn.line_items(&mut conn)?;
        let li_json: Vec<Value> = line_items
            .iter()
            .map(|li| {
                json!({
                    "id": li.id,
                    "category_id": li.category_id,
                    "amount": li.amount,
                    "note": li.note,
                })
            })
            .collect();

        items.push(json!({
            "id": txn.id,
            "account_id": txn.account_id,
            "transfer_account_id": txn.transfer_account_id,
            "date": txn.date,
            "description": txn.description,
            "amount": txn.amount,
            "cleared": txn.cleared,
            "line_items": li_json,
        }));
    }

    Ok(Json(json!({ "transactions": items })))
}

#[derive(Deserialize)]
pub struct LineItemRequest {
    category_id: String,
    amount: i32,
    note: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    account_id: AccountRef,
    transfer_account_id: Option<AccountRef>,
    date: String,
    description: Option<String>,
    amount: i32,
    cleared: Option<bool>,
    line_items: Vec<LineItemRequest>,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateTransactionRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let acct = Account::from_ref(&body.account_id, &mut conn)?
        .ok_or(AppError::NotFound("account".into()))?;

    if acct.workspace_id != member.workspace.id {
        return Err(AppError::NotFound("account".into()));
    }

    let line_items: Vec<LineItemInput> = body
        .line_items
        .into_iter()
        .map(|li| {
            let mut input = LineItemInput {
                category_id: li.category_id,
                amount: li.amount,
                note: None,
            };
            if let Some(note) = li.note {
                input.note = Some(note);
            }
            input
        })
        .collect();

    let mut builder = NewTransaction::new(
        &member.workspace,
        &acct,
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
    if let Some(ref transfer_id) = body.transfer_account_id {
        let transfer_acct = Account::from_ref(transfer_id, &mut conn)?
            .ok_or(AppError::NotFound("transfer account".into()))?;
        if transfer_acct.workspace_id != member.workspace.id {
            return Err(AppError::NotFound("transfer account".into()));
        }
        builder = builder.transfer_to(&transfer_acct);
    }

    let txn = builder.insert(&mut conn)?;

    let items = txn.line_items(&mut conn)?;
    let li_json: Vec<Value> = items
        .iter()
        .map(|li| {
            json!({
                "id": li.id,
                "category_id": li.category_id,
                "amount": li.amount,
                "note": li.note,
            })
        })
        .collect();

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({
            "id": txn.id,
            "account_id": txn.account_id,
            "transfer_account_id": txn.transfer_account_id,
            "date": txn.date,
            "description": txn.description,
            "amount": txn.amount,
            "cleared": txn.cleared,
            "line_items": li_json,
        })),
    ))
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path((_workspace_id, transaction_id)): Path<(String, TransactionRef)>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let txn = Transaction::from_ref(&transaction_id, &mut conn)?
        .ok_or(AppError::NotFound("transaction".into()))?;

    if txn.workspace_id != member.workspace.id {
        return Err(AppError::NotFound("transaction".into()));
    }

    txn.soft_delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
