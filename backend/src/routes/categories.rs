use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::category::{Category, CategoryRef, NewCategory};
use crate::domain::workspace::Role;
use crate::error::AppError;
use crate::DbPool;

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let categories = Category::for_workspace(&member.workspace, &mut conn)?;

    let items: Vec<Value> = categories
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "description": c.description,
            })
        })
        .collect();

    Ok(Json(json!({ "categories": items })))
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    name: String,
    description: Option<String>,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<CreateCategoryRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let mut builder = NewCategory::new(&member.workspace, &body.name);
    if let Some(ref desc) = body.description {
        builder = builder.description(desc);
    }
    let cat = builder.insert(&mut conn)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({
            "id": cat.id,
            "name": cat.name,
            "description": cat.description,
        })),
    ))
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path((_workspace_id, category_id)): Path<(String, CategoryRef)>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let cat = Category::from_ref(&category_id, &mut conn)?
        .ok_or(AppError::NotFound("category".into()))?;

    if cat.workspace_id != member.workspace.id {
        return Err(AppError::NotFound("category".into()));
    }

    cat.soft_delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
