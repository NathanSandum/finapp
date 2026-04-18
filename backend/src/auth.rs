use axum::{
    extract::{FromRequestParts, Path},
    http::{header::AUTHORIZATION, request::Parts},
};
use diesel::prelude::*;

use crate::domain::session::Session;
use crate::domain::user::User;
use crate::domain::workspace::{Role, Workspace};
use crate::error::AppError;
use crate::schema::workspaces;
use crate::DbPool;

/// Extracts the authenticated user from the Bearer token.
pub struct AuthUser(pub User);

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let pool = parts
            .extensions
            .get::<DbPool>()
            .ok_or_else(|| AppError::Internal("missing db pool".into()))?;

        let mut conn = pool.get().map_err(|err| {
            AppError::Internal(format!("db pool error: {err}"))
        })?;

        Session::validate(token, &mut conn)?
            .map(AuthUser)
            .ok_or(AppError::Unauthorized)
    }
}

/// Extracts the authenticated user + workspace, enforcing membership and minimum role.
pub struct WorkspaceMember {
    pub user: User,
    pub workspace: Workspace,
    pub role: Role,
}

impl<S: Send + Sync> FromRequestParts<S> for WorkspaceMember {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;

        let workspace_id = parts
            .extensions
            .get::<Path<WorkspacePathParam>>()
            .map(|p| p.workspace_id.clone());

        // Fall back to extracting from URI path segments
        let workspace_id = match workspace_id {
            Some(id) => id,
            None => extract_workspace_id_from_uri(parts)?,
        };

        let pool = parts
            .extensions
            .get::<DbPool>()
            .ok_or_else(|| AppError::Internal("missing db pool".into()))?;

        let mut conn = pool.get().map_err(|err| {
            AppError::Internal(format!("db pool error: {err}"))
        })?;

        let workspace = workspaces::table
            .find(&workspace_id)
            .select(Workspace::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("workspace lookup failed: {err}")))?
            .ok_or(AppError::Forbidden)?;

        let role = workspace
            .user_role(&user, &mut conn)?
            .ok_or(AppError::Forbidden)?;

        Ok(WorkspaceMember {
            user,
            workspace,
            role,
        })
    }
}

#[derive(serde::Deserialize)]
struct WorkspacePathParam {
    workspace_id: String,
}

fn extract_workspace_id_from_uri(parts: &Parts) -> Result<String, AppError> {
    let path = parts.uri.path();
    let segments: Vec<&str> = path.split('/').collect();

    // Look for /workspaces/{id} pattern
    for (i, seg) in segments.iter().enumerate() {
        if *seg == "workspaces" {
            if let Some(id) = segments.get(i + 1) {
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }
    }

    Err(AppError::Internal("workspace_id not found in path".into()))
}

impl WorkspaceMember {
    /// Check that the member has at least the required role.
    pub fn require_role(&self, minimum: &Role) -> Result<(), AppError> {
        let level = |r: &Role| match r {
            Role::Owner => 3,
            Role::Editor => 2,
            Role::Viewer => 1,
        };

        if level(&self.role) >= level(minimum) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_role_owner_passes_all() {
        let member = WorkspaceMember {
            user: stub_user(),
            workspace: stub_workspace(),
            role: Role::Owner,
        };
        assert!(member.require_role(&Role::Viewer).is_ok());
        assert!(member.require_role(&Role::Editor).is_ok());
        assert!(member.require_role(&Role::Owner).is_ok());
    }

    #[test]
    fn require_role_editor_fails_owner() {
        let member = WorkspaceMember {
            user: stub_user(),
            workspace: stub_workspace(),
            role: Role::Editor,
        };
        assert!(member.require_role(&Role::Viewer).is_ok());
        assert!(member.require_role(&Role::Editor).is_ok());
        assert!(member.require_role(&Role::Owner).is_err());
    }

    #[test]
    fn require_role_viewer_only_passes_viewer() {
        let member = WorkspaceMember {
            user: stub_user(),
            workspace: stub_workspace(),
            role: Role::Viewer,
        };
        assert!(member.require_role(&Role::Viewer).is_ok());
        assert!(member.require_role(&Role::Editor).is_err());
        assert!(member.require_role(&Role::Owner).is_err());
    }

    fn stub_user() -> User {
        User {
            id: "user-1".into(),
            email: "test@example.com".into(),
            display_name: "Test".into(),
            password_hash: "hash".into(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    fn stub_workspace() -> Workspace {
        Workspace {
            id: "ws-1".into(),
            name: "Test WS".into(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}
