use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::AccountRef;
use crate::domain::entry::{Entry, EntryRef, NewEntry, PostingInput, UpdateEntry};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::DbPool;

#[derive(Deserialize)]
pub struct CreatePostingRequest {
    account_id: AccountRef,
    amount: i32,
    note: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateEntryRequest {
    date: String,
    description: String,
    #[serde(default)]
    cleared: bool,
    postings: Vec<CreatePostingRequest>,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateEntryRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let posting_inputs: Vec<PostingInput> = body
        .postings
        .into_iter()
        .map(|p| {
            let mut input = PostingInput::new(p.account_id, p.amount);
            if let Some(note) = p.note {
                input = input.note(&note);
            }
            input
        })
        .collect();

    let entry = NewEntry::new(
        member.workspace.clone(),
        &body.date,
        &body.description,
        posting_inputs,
    )
    .cleared(body.cleared)
    .insert(&mut conn)?;

    let postings = entry.postings(&mut conn)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(entry_to_json(&entry, &postings)),
    ))
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entries = member.workspace.entries(&mut conn)?;

    let mut items = Vec::with_capacity(entries.len());
    for entry in &entries {
        let postings = entry.postings(&mut conn)?;
        items.push(entry_to_json(entry, &postings));
    }

    Ok(Json(json!({ "entries": items })))
}

#[derive(Deserialize)]
pub struct EntryPath {
    #[allow(dead_code)]
    workspace: WorkspaceRef,
    entry: EntryRef,
}

pub async fn get(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<EntryPath>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entry =
        Entry::from_ref(&path.entry, &mut conn)?.ok_or(AppError::NotFound("entry".into()))?;

    if entry.workspace != member.workspace {
        return Err(AppError::NotFound("entry".into()));
    }

    let postings = entry.postings(&mut conn)?;

    Ok(Json(entry_to_json(&entry, &postings)))
}

#[derive(Deserialize)]
pub struct UpdateEntryRequest {
    date: Option<String>,
    description: Option<String>,
    cleared: Option<bool>,
    postings: Option<Vec<CreatePostingRequest>>,
}

pub async fn update(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<EntryPath>,
    Json(body): Json<UpdateEntryRequest>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entry =
        Entry::from_ref(&path.entry, &mut conn)?.ok_or(AppError::NotFound("entry".into()))?;

    if entry.workspace != member.workspace {
        return Err(AppError::NotFound("entry".into()));
    }

    let posting_inputs = body.postings.map(|postings| {
        postings
            .into_iter()
            .map(|p| {
                let mut input = PostingInput::new(p.account_id, p.amount);
                if let Some(note) = p.note {
                    input = input.note(&note);
                }
                input
            })
            .collect()
    });

    let changes = UpdateEntry {
        date: body.date,
        description: body.description,
        cleared: body.cleared,
        postings: posting_inputs,
    };

    let updated = entry.update(changes, &mut conn)?;
    let postings = updated.postings(&mut conn)?;

    Ok(Json(entry_to_json(&updated, &postings)))
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<EntryPath>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entry =
        Entry::from_ref(&path.entry, &mut conn)?.ok_or(AppError::NotFound("entry".into()))?;

    if entry.workspace != member.workspace {
        return Err(AppError::NotFound("entry".into()));
    }

    entry.soft_delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}

pub fn entry_to_json(
    entry: &Entry,
    postings: &[crate::domain::entry::Posting],
) -> Value {
    let posting_items: Vec<Value> = postings
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "account_id": p.account,
                "amount": p.amount,
                "note": p.note,
            })
        })
        .collect();

    json!({
        "id": entry.id,
        "date": entry.date,
        "description": entry.description,
        "cleared": entry.cleared,
        "postings": posting_items,
    })
}
