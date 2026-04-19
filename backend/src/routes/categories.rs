use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::category::{Category, CategoryRef, NewCategory};
use crate::domain::workspace::{Role, WorkspaceRef};
use crate::error::AppError;
use crate::DbPool;

pub async fn list(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let categories = member.workspace.categories(&mut conn)?;

    Ok(Json(json!({ "categories": categories })))
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
) -> Result<(axum::http::StatusCode, Json<Category>), AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let mut builder = NewCategory::new(&member.workspace, &body.name);
    if let Some(ref desc) = body.description {
        builder = builder.description(desc);
    }
    let category = builder.insert(&mut conn)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(category),
    ))
}

#[derive(Deserialize)]
pub struct DeleteCategoryPath {
    workspace: WorkspaceRef,
    category: CategoryRef,
}

pub async fn delete(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Path(path): Path<DeleteCategoryPath>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let category = Category::from_ref(&path.category, &mut conn)?
        .ok_or(AppError::NotFound("category".into()))?;

    if category.workspace != member.workspace {
        return Err(AppError::NotFound("category".into()));
    }

    category.soft_delete(&mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
