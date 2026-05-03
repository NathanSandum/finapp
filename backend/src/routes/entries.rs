use axum::{extract::Extension, extract::Path, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::AccountRef;
use crate::domain::entry::{Entry, EntryRef, NewEntry, PostingInput, UpdateEntry};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::jsend::{JSend, JSendResponse};
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
    postings: Vec<CreatePostingRequest>,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateEntryRequest>,
) -> Result<JSendResponse<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let posting_inputs: Vec<PostingInput> = body
        .postings
        .into_iter()
        .map(posting_request_to_input)
        .collect();

    let entry = NewEntry::new(
        member.workspace.clone(),
        &body.date,
        &body.description,
        posting_inputs,
    )
    .insert(&mut conn)?;

    let postings = entry.postings(&mut conn)?;

    Ok(JSendResponse::new(
        StatusCode::CREATED,
        JSend::success(entry_to_json(&entry, &postings)),
    ))
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<JSend<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entries = member.workspace.entries(&mut conn)?;

    let mut items = Vec::with_capacity(entries.len());
    for entry in &entries {
        let postings = entry.postings(&mut conn)?;
        items.push(entry_to_json(entry, &postings));
    }

    Ok(JSend::success(json!({ "entries": items })))
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
) -> Result<JSend<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entry =
        Entry::from_ref(&path.entry, &mut conn)?.ok_or(AppError::NotFound("entry".into()))?;

    if entry.workspace != member.workspace {
        return Err(AppError::NotFound("entry".into()));
    }

    let postings = entry.postings(&mut conn)?;

    Ok(JSend::success(entry_to_json(&entry, &postings)))
}

#[derive(Deserialize)]
pub struct UpdateEntryRequest {
    date: Option<String>,
    description: Option<String>,
    postings: Option<Vec<CreatePostingRequest>>,
}

pub async fn update(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<EntryPath>,
    Json(body): Json<UpdateEntryRequest>,
) -> Result<JSend<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let entry =
        Entry::from_ref(&path.entry, &mut conn)?.ok_or(AppError::NotFound("entry".into()))?;

    if entry.workspace != member.workspace {
        return Err(AppError::NotFound("entry".into()));
    }

    let posting_inputs = body
        .postings
        .map(|postings| postings.into_iter().map(posting_request_to_input).collect());

    let changes = UpdateEntry {
        date: body.date,
        description: body.description,
        postings: posting_inputs,
    };

    let updated = entry.update(changes, &mut conn)?;
    let postings = updated.postings(&mut conn)?;

    Ok(JSend::success(entry_to_json(&updated, &postings)))
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<EntryPath>,
) -> Result<JSend<Value>, AppError> {
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

    Ok(JSend::success(Value::Null))
}

#[derive(Deserialize)]
pub struct MergeEntriesRequest {
    source_entry_ids: Vec<EntryRef>,
    date: String,
    description: String,
    postings: Vec<CreatePostingRequest>,
}

pub async fn merge(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<MergeEntriesRequest>,
) -> Result<JSendResponse<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let mut source_entries = Vec::with_capacity(body.source_entry_ids.len());
    for entry_ref in &body.source_entry_ids {
        let entry =
            Entry::from_ref(entry_ref, &mut conn)?.ok_or(AppError::NotFound("entry".into()))?;
        if entry.workspace != member.workspace {
            return Err(AppError::NotFound("entry".into()));
        }
        source_entries.push(entry);
    }

    let posting_inputs: Vec<PostingInput> = body
        .postings
        .into_iter()
        .map(posting_request_to_input)
        .collect();

    let entry = Entry::merge(
        &source_entries,
        &member.workspace,
        &body.date,
        &body.description,
        posting_inputs,
        &mut conn,
    )?;

    let postings = entry.postings(&mut conn)?;

    Ok(JSendResponse::new(
        StatusCode::CREATED,
        JSend::success(entry_to_json(&entry, &postings)),
    ))
}

fn posting_request_to_input(request: CreatePostingRequest) -> PostingInput {
    let mut input = PostingInput::new(request.account_id, request.amount);
    if let Some(note) = request.note {
        input = input.note(&note);
    }
    input
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
        "postings": posting_items,
    })
}
