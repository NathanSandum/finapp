use axum::{extract::Extension, extract::Path, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::AuthUser;
use crate::domain::user::User;
use crate::domain::workspace::{NewWorkspace, Role, Workspace};
use crate::error::AppError;
use crate::DbPool;

#[derive(Deserialize)]
pub struct CreateWorkspaceRequest {
    name: String,
}

pub async fn create(
    Extension(pool): Extension<DbPool>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let ws = NewWorkspace::new(&body.name, &user)
        .insert(&mut conn)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({
            "id": ws.id,
            "name": ws.name,
            "created_at": ws.created_at.to_string(),
        })),
    ))
}

pub async fn list(
    Extension(pool): Extension<DbPool>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let workspaces = Workspace::for_user(&user, &mut conn)?;

    let items: Vec<Value> = workspaces
        .iter()
        .map(|ws| {
            json!({
                "id": ws.id,
                "name": ws.name,
                "created_at": ws.created_at.to_string(),
            })
        })
        .collect();

    Ok(Json(json!({ "workspaces": items })))
}

pub async fn get(
    Extension(pool): Extension<DbPool>,
    AuthUser(user): AuthUser,
    Path(workspace_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let ws = Workspace::from_id(&workspace_id, &mut conn)?
        .ok_or(AppError::Forbidden)?;

    // Non-member gets 403 (indistinguishable from not-found per spec)
    let role = ws.user_role(&user, &mut conn)?
        .ok_or(AppError::Forbidden)?;

    let members = ws.members(&mut conn)?;
    let member_list: Vec<Value> = members
        .iter()
        .map(|m| {
            json!({
                "user_id": m.user_id,
                "role": m.role,
                "joined_at": m.joined_at.to_string(),
            })
        })
        .collect();

    Ok(Json(json!({
        "id": ws.id,
        "name": ws.name,
        "created_at": ws.created_at.to_string(),
        "your_role": role.as_str(),
        "members": member_list,
    })))
}

#[derive(Deserialize)]
pub struct RemoveMemberPath {
    workspace_id: String,
    user_id: String,
}

pub async fn remove_member(
    Extension(pool): Extension<DbPool>,
    AuthUser(actor): AuthUser,
    Path(path): Path<RemoveMemberPath>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let ws = Workspace::from_id(&path.workspace_id, &mut conn)?
        .ok_or(AppError::Forbidden)?;

    // Only owners can remove members (unless removing self)
    let actor_role = ws.user_role(&actor, &mut conn)?
        .ok_or(AppError::Forbidden)?;

    if path.user_id != actor.id && actor_role < Role::Owner {
        return Err(AppError::Forbidden);
    }

    let target = User::from_id(&path.user_id, &mut conn)?
        .ok_or(AppError::NotFound("user".into()))?;

    ws.remove_member(&target, &actor, &mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
