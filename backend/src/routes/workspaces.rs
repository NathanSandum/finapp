use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::AuthUser;
use crate::domain::user::{User, UserRef};
use crate::domain::workspace::{NewWorkspace, Workspace, WorkspaceRef};
use crate::error::AppError;
use crate::permissions;
use crate::DbPool;

#[derive(Deserialize)]
pub struct CreateWorkspaceRequest {
    name: String,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<(axum::http::StatusCode, Json<Workspace>), AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let workspace = NewWorkspace::new(&body.name, user.as_ref()).insert(&mut conn)?;

    Ok((axum::http::StatusCode::CREATED, Json(workspace)))
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let workspaces = user.as_ref().workspaces(&mut conn)?;
    Ok(Json(json!({"workspaces": workspaces})))
}

pub async fn get(
    Path(workspace): Path<WorkspaceRef>,
    AuthUser(user): AuthUser,
    Extension(pool): Extension<DbPool>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let workspace = Workspace::from_ref(&workspace, &mut conn)?.ok_or(AppError::Forbidden)?;

    let user = workspace
        .view_user(user.as_ref(), &mut conn)?
        .ok_or(AppError::Forbidden)?;

    let members = workspace.members(&mut conn)?;

    Ok(Json(json!({
        "workspace": workspace.id,
        "name": workspace.name,
        "created_at": workspace.created_at.to_string(),
        "your_role": user.role.as_str(),
        "members": members,
    })))
}

#[derive(Deserialize)]
pub struct RemoveMemberPath {
    workspace: WorkspaceRef,
    user: UserRef,
}

pub async fn remove_member(
    Path(path): Path<RemoveMemberPath>,
    AuthUser(actor): AuthUser,
    Extension(pool): Extension<DbPool>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool
        .get()
        .map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let workspace = Workspace::from_ref(&path.workspace, &mut conn)?.ok_or(AppError::Forbidden)?;

    let actor_role = workspace
        .view_user(actor.as_ref(), &mut conn)?
        .ok_or(AppError::Forbidden)?
        .role;

    let target = User::from_ref(&path.user, &mut conn)?.ok_or(AppError::NotFound("user".into()))?;

    let target_role = workspace
        .view_user(target.as_ref(), &mut conn)?
        .ok_or(AppError::NotFound("member".into()))?
        .role;

    let resource =
        permissions::membership_resource(&actor.as_ref(), &target.as_ref(), &target_role);

    if !actor_role.has_permission(permissions::Action::Delete, resource) {
        return Err(AppError::Forbidden);
    }

    workspace.remove_member(target.as_ref(), actor.as_ref(), &mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
