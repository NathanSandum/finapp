use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::AppError;
use crate::schema::{membership_events, workspace_members, workspaces};

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = workspaces)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Workspace {
    pub fn find_by_id(
        conn: &mut SqliteConnection,
        id: &str,
    ) -> Result<Option<Workspace>, AppError> {
        workspaces::table
            .find(id)
            .select(Workspace::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("workspace lookup failed: {err}")))
    }

    pub fn find_all_for_user(
        conn: &mut SqliteConnection,
        user_id: &str,
    ) -> Result<Vec<Workspace>, AppError> {
        workspace_members::table
            .filter(workspace_members::user_id.eq(user_id))
            .inner_join(workspaces::table.on(workspaces::id.eq(workspace_members::workspace_id)))
            .select(Workspace::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("find workspaces for user failed: {err}")))
    }

    pub fn role_for_user(
        conn: &mut SqliteConnection,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<Option<Role>, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .filter(workspace_members::user_id.eq(user_id))
            .select(workspace_members::role)
            .first::<String>(conn)
            .optional()
            .map(|r| r.and_then(|s| Role::from_str(&s)))
            .map_err(|err| AppError::Internal(format!("role lookup failed: {err}")))
    }

    pub fn owner_count(
        conn: &mut SqliteConnection,
        workspace_id: &str,
    ) -> Result<i64, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .filter(workspace_members::role.eq(Role::Owner.as_str()))
            .count()
            .get_result(conn)
            .map_err(|err| AppError::Internal(format!("owner count query failed: {err}")))
    }

    pub fn members(
        conn: &mut SqliteConnection,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceMember>, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .select(WorkspaceMember::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("members query failed: {err}")))
    }

    pub fn is_member(
        conn: &mut SqliteConnection,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<bool, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .filter(workspace_members::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
            .map(|count| count > 0)
            .map_err(|err| AppError::Internal(format!("is_member query failed: {err}")))
    }

    pub fn remove_member(
        conn: &mut SqliteConnection,
        workspace_id: &str,
        user_id: &str,
        actor_id: &str,
    ) -> Result<(), AppError> {
        conn.transaction::<_, AppError, _>(|conn| {
            let role = workspace_members::table
                .filter(workspace_members::workspace_id.eq(workspace_id))
                .filter(workspace_members::user_id.eq(user_id))
                .select(workspace_members::role)
                .first::<String>(conn)
                .optional()?
                .ok_or_else(|| AppError::NotFound("member".into()))?;

            if role == Role::Owner.as_str() {
                let owner_count: i64 = workspace_members::table
                    .filter(workspace_members::workspace_id.eq(workspace_id))
                    .filter(workspace_members::role.eq(Role::Owner.as_str()))
                    .count()
                    .get_result(conn)?;

                if owner_count <= 1 {
                    return Err(AppError::Conflict(
                        "cannot remove the last owner of a workspace".into(),
                    ));
                }
            }

            let now = chrono::Utc::now().naive_utc();
            let event_type = if user_id == actor_id { "left" } else { "removed" };

            diesel::insert_into(membership_events::table)
                .values(NewMembershipEvent {
                    id: Uuid::new_v4().to_string(),
                    workspace_id: workspace_id.to_string(),
                    user_id: user_id.to_string(),
                    actor_id: actor_id.to_string(),
                    event_type: event_type.to_string(),
                    role: None,
                    created_at: now,
                })
                .execute(conn)?;

            diesel::delete(
                workspace_members::table
                    .filter(workspace_members::workspace_id.eq(workspace_id))
                    .filter(workspace_members::user_id.eq(user_id)),
            )
            .execute(conn)?;

            Ok(())
        })
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = workspace_members)]
pub struct WorkspaceMember {
    pub workspace_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: NaiveDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = membership_events)]
pub struct MembershipEvent {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub actor_id: String,
    pub event_type: String,
    pub role: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = workspaces)]
struct NewWorkspaceRow {
    id: String,
    name: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = membership_events)]
struct NewMembershipEvent {
    id: String,
    workspace_id: String,
    user_id: String,
    actor_id: String,
    event_type: String,
    role: Option<String>,
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = workspace_members)]
struct NewWorkspaceMember {
    workspace_id: String,
    user_id: String,
    role: String,
    joined_at: NaiveDateTime,
}

// ── NewWorkspace ────────────────────────────────────────────────────────────

pub struct NewWorkspace {
    name: String,
    creator_id: String,
}

impl NewWorkspace {
    pub fn new(name: &str, creator_id: &str) -> Self {
        NewWorkspace {
            name: name.to_string(),
            creator_id: creator_id.to_string(),
        }
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<Workspace, AppError> {
        conn.transaction::<_, AppError, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();
            let workspace_id = Uuid::new_v4().to_string();

            let workspace = diesel::insert_into(workspaces::table)
                .values(NewWorkspaceRow {
                    id: workspace_id.clone(),
                    name: self.name,
                    created_at: now,
                    updated_at: now,
                })
                .returning(Workspace::as_returning())
                .get_result(conn)?;

            diesel::insert_into(membership_events::table)
                .values(NewMembershipEvent {
                    id: Uuid::new_v4().to_string(),
                    workspace_id: workspace_id.clone(),
                    user_id: self.creator_id.clone(),
                    actor_id: self.creator_id.clone(),
                    event_type: "owner_assigned".to_string(),
                    role: Some(Role::Owner.as_str().to_string()),
                    created_at: now,
                })
                .execute(conn)?;

            diesel::insert_into(workspace_members::table)
                .values(NewWorkspaceMember {
                    workspace_id: workspace_id.clone(),
                    user_id: self.creator_id,
                    role: Role::Owner.as_str().to_string(),
                    joined_at: now,
                })
                .execute(conn)?;

            Ok(workspace)
        })
    }
}

// ── Role ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Owner,
    Editor,
    Viewer,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Editor => "editor",
            Role::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(Role::Owner),
            "editor" => Some(Role::Editor),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::NewUser;
    use crate::test_helpers::setup_test_db;

    fn make_user(conn: &mut SqliteConnection, email: &str) -> String {
        NewUser::new(email, "Test User", "password")
            .expect("new user should succeed")
            .insert(conn)
            .expect("user insert should succeed")
            .id
    }

    fn make_workspace(conn: &mut SqliteConnection, name: &str, creator_id: &str) -> Workspace {
        NewWorkspace::new(name, creator_id)
            .insert(conn)
            .expect("workspace creation should succeed")
    }

    #[test]
    fn create_workspace_succeeds() {
        let mut conn = setup_test_db();
        let user_id = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user_id);
        assert_eq!(ws.name, "My Finances");
        assert!(!ws.id.is_empty());
    }

    #[test]
    fn creator_is_assigned_owner() {
        let mut conn = setup_test_db();
        let user_id = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user_id);

        let role = Workspace::role_for_user(&mut conn, &ws.id, &user_id)
            .expect("role query should succeed")
            .expect("creator should have a role");

        assert_eq!(role, Role::Owner);
    }

    #[test]
    fn user_can_belong_to_multiple_workspaces() {
        let mut conn = setup_test_db();
        let user_id = make_user(&mut conn, "alice@example.com");
        make_workspace(&mut conn, "Workspace A", &user_id);
        make_workspace(&mut conn, "Workspace B", &user_id);

        let workspaces = Workspace::find_all_for_user(&mut conn, &user_id)
            .expect("find_all should succeed");
        assert_eq!(workspaces.len(), 2);
    }

    #[test]
    fn owner_event_is_appended_to_log() {
        let mut conn = setup_test_db();
        let user_id = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user_id);

        let events: Vec<MembershipEvent> = membership_events::table
            .filter(membership_events::workspace_id.eq(&ws.id))
            .select(MembershipEvent::as_select())
            .load(&mut conn)
            .expect("event query should succeed");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "owner_assigned");
        assert_eq!(events[0].role, Some("owner".to_string()));
    }

    #[test]
    fn remove_last_owner_is_rejected_atomically() {
        let mut conn = setup_test_db();
        let user_id = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user_id);

        let err = Workspace::remove_member(&mut conn, &ws.id, &user_id, &user_id)
            .expect_err("removing last owner should fail");

        assert!(matches!(err, AppError::Conflict(_)));

        let count = Workspace::owner_count(&mut conn, &ws.id)
            .expect("owner count should succeed");
        assert_eq!(count, 1, "workspace must still have one owner");
    }

    #[test]
    fn non_member_has_no_role() {
        let mut conn = setup_test_db();
        let owner_id = make_user(&mut conn, "owner@example.com");
        let outsider_id = make_user(&mut conn, "outsider@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &owner_id);

        let role = Workspace::role_for_user(&mut conn, &ws.id, &outsider_id)
            .expect("role query should succeed");
        assert!(role.is_none(), "outsider should have no role");
    }

    #[test]
    fn is_member_returns_false_for_non_member() {
        let mut conn = setup_test_db();
        let owner_id = make_user(&mut conn, "owner@example.com");
        let outsider_id = make_user(&mut conn, "outsider@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &owner_id);

        assert!(!Workspace::is_member(&mut conn, &ws.id, &outsider_id)
            .expect("is_member should succeed"));
        assert!(Workspace::is_member(&mut conn, &ws.id, &owner_id)
            .expect("is_member should succeed"));
    }

    #[test]
    fn workspace_data_is_scoped_by_workspace_id() {
        let mut conn = setup_test_db();
        let user_a = make_user(&mut conn, "a@example.com");
        let user_b = make_user(&mut conn, "b@example.com");
        let ws_a = make_workspace(&mut conn, "Workspace A", &user_a);
        let ws_b = make_workspace(&mut conn, "Workspace B", &user_b);

        let members_b = Workspace::members(&mut conn, &ws_b.id)
            .expect("members query should succeed");
        assert!(
            members_b.iter().all(|m| m.user_id != user_a),
            "user_a must not appear in workspace B"
        );

        let members_a = Workspace::members(&mut conn, &ws_a.id)
            .expect("members query should succeed");
        assert!(
            members_a.iter().all(|m| m.user_id != user_b),
            "user_b must not appear in workspace A"
        );
    }

    #[test]
    fn find_all_for_user_returns_only_their_workspaces() {
        let mut conn = setup_test_db();
        let user_a = make_user(&mut conn, "a@example.com");
        let user_b = make_user(&mut conn, "b@example.com");
        make_workspace(&mut conn, "A's Workspace", &user_a);
        make_workspace(&mut conn, "B's Workspace", &user_b);

        let a_workspaces = Workspace::find_all_for_user(&mut conn, &user_a)
            .expect("find_all_for_user should succeed");

        assert_eq!(a_workspaces.len(), 1);
        assert_eq!(a_workspaces[0].name, "A's Workspace");
    }

    #[test]
    fn non_owner_can_be_removed() {
        let mut conn = setup_test_db();
        let owner_id = make_user(&mut conn, "owner@example.com");
        let editor_id = make_user(&mut conn, "editor@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &owner_id);

        diesel::insert_into(workspace_members::table)
            .values(NewWorkspaceMember {
                workspace_id: ws.id.clone(),
                user_id: editor_id.clone(),
                role: Role::Editor.as_str().to_string(),
                joined_at: chrono::Utc::now().naive_utc(),
            })
            .execute(&mut conn)
            .expect("insert editor should succeed");

        Workspace::remove_member(&mut conn, &ws.id, &editor_id, &owner_id)
            .expect("removing editor should succeed");

        let role = Workspace::role_for_user(&mut conn, &ws.id, &editor_id)
            .expect("role query should succeed");
        assert!(role.is_none(), "editor should no longer be a member");
    }
}
