use axum::{extract::FromRequestParts, http::request::Parts};

use crate::auth_backend::AuthBackend;
use crate::domain::user::{User, UserRef};
use crate::domain::workspace::{Role, Workspace, WorkspaceRef};
use crate::error::AppError;
use crate::DbPool;

type AuthSession = axum_login::AuthSession<AuthBackend>;

/// Extracts the authenticated user (without workspace context).
pub struct AuthUser(pub User);

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Internal("auth layer missing".into()))?;
        let user = auth_session.user.ok_or(AppError::Unauthorized)?;
        Ok(AuthUser(user))
    }
}

/// Extracts the authenticated user + workspace, enforcing membership and minimum role.
pub struct WorkspaceMember {
    pub user: UserRef,
    pub workspace: WorkspaceRef,
    pub role: Role,
}

impl WorkspaceMember {
    pub fn require_role(&self, minimum: &Role) -> Result<(), AppError> {
        if self.role >= *minimum {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

fn extract_workspace_id_from_uri(parts: &Parts) -> Result<String, AppError> {
    let path = parts.uri.path();
    let segments: Vec<&str> = path.split('/').collect();

    for (i, segment) in segments.iter().enumerate() {
        if *segment == "workspaces" {
            if let Some(id) = segments.get(i + 1) {
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }
    }

    Err(AppError::Internal("workspace_id not found in path".into()))
}

impl<S: Send + Sync> FromRequestParts<S> for WorkspaceMember {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Internal("auth layer missing".into()))?;
        let user = auth_session.user.ok_or(AppError::Unauthorized)?;
        let workspace_id = extract_workspace_id_from_uri(parts)?;

        let db = parts
            .extensions
            .get::<DbPool>()
            .ok_or_else(|| AppError::Internal("missing db".into()))?;

        let mut conn = db
            .get()
            .map_err(|err| AppError::Internal(format!("db: {err}")))?;

        let workspace_ref: WorkspaceRef =
            serde_json::from_value(serde_json::json!(workspace_id))
                .map_err(|_| AppError::Internal("invalid workspace id".into()))?;

        let workspace =
            Workspace::from_ref(&workspace_ref, &mut conn)?.ok_or(AppError::Forbidden)?;

        let member = workspace
            .view_user(user.as_ref(), &mut conn)?
            .ok_or(AppError::Forbidden)?;

        Ok(WorkspaceMember {
            user: user.as_ref().clone(),
            workspace: workspace.as_ref().clone(),
            role: member.role,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::UserRef;

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

    fn stub_user() -> UserRef {
        serde_json::from_value(serde_json::json!("user-1")).unwrap()
    }

    fn stub_workspace() -> WorkspaceRef {
        serde_json::from_value(serde_json::json!("ws-1")).unwrap()
    }
}
