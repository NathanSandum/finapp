use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::user::User;
use crate::error::AppError;
use crate::schema::{membership_events, workspace_members, workspaces};

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct WorkspaceRef(String);


#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = workspaces)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Workspace {
    pub fn as_ref(&self) -> WorkspaceRef {
        WorkspaceRef(self.id.clone())
    }

    pub fn from_ref(
        workspace_ref: &WorkspaceRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Workspace>, AppError> {
        workspaces::table
            .find(&workspace_ref.0)
            .select(Workspace::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("workspace lookup failed: {err}")))
    }

    pub fn for_user(
        user: &User,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Workspace>, AppError> {
        workspace_members::table
            .filter(workspace_members::user_id.eq(&user.id))
            .inner_join(workspaces::table.on(workspaces::id.eq(workspace_members::workspace_id)))
            .select(Workspace::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("find workspaces for user failed: {err}")))
    }

    pub fn user_role(
        &self,
        user: &User,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Role>, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .filter(workspace_members::user_id.eq(&user.id))
            .select(workspace_members::role)
            .first::<String>(conn)
            .optional()
            .map(|r| r.and_then(|s| Role::from_str(&s)))
            .map_err(|err| AppError::Internal(format!("role lookup failed: {err}")))
    }

    pub fn num_owners(&self, conn: &mut SqliteConnection) -> Result<i64, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .filter(workspace_members::role.eq(Role::Owner.as_str()))
            .count()
            .get_result(conn)
            .map_err(|err| AppError::Internal(format!("owner count query failed: {err}")))
    }

    pub fn members(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<WorkspaceMember>, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .select(WorkspaceMember::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("members query failed: {err}")))
    }

    pub fn is_member(
        &self,
        user: &User,
        conn: &mut SqliteConnection,
    ) -> Result<bool, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .filter(workspace_members::user_id.eq(&user.id))
            .count()
            .get_result::<i64>(conn)
            .map(|count| count > 0)
            .map_err(|err| AppError::Internal(format!("is_member query failed: {err}")))
    }

    pub fn remove_member(
        &self,
        user: &User,
        actor: &User,
        conn: &mut SqliteConnection,
    ) -> Result<(), AppError> {
        conn.transaction::<_, AppError, _>(|conn| {
            let role = workspace_members::table
                .filter(workspace_members::workspace_id.eq(&self.id))
                .filter(workspace_members::user_id.eq(&user.id))
                .select(workspace_members::role)
                .first::<String>(conn)
                .optional()?
                .ok_or_else(|| AppError::NotFound("member".into()))?;

            if role == Role::Owner.as_str() {
                let owner_count: i64 = workspace_members::table
                    .filter(workspace_members::workspace_id.eq(&self.id))
                    .filter(workspace_members::role.eq(Role::Owner.as_str()))
                    .count()
                    .get_result(conn)?;

                if owner_count <= 1 {
                    return Err(AppError::Conflict(
                        "cannot remove the last owner of a workspace".into(),
                    ));
                }
            }

            let event_type = if user.id == actor.id { "left" } else { "removed" };

            NewMembershipEvent::new(&self.id, &user.id, &actor.id, event_type, None)
                .insert(conn)?;

            diesel::delete(
                workspace_members::table
                    .filter(workspace_members::workspace_id.eq(&self.id))
                    .filter(workspace_members::user_id.eq(&user.id)),
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

// ── Insertables ──────────────────────────────────────────────────────────────

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

impl NewMembershipEvent {
    fn new(
        workspace_id: &str,
        user_id: &str,
        actor_id: &str,
        event_type: &str,
        role: Option<&str>,
    ) -> Self {
        NewMembershipEvent {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
            actor_id: actor_id.to_string(),
            event_type: event_type.to_string(),
            role: role.map(|r| r.to_string()),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    fn insert(self, conn: &mut SqliteConnection) -> Result<(), AppError> {
        diesel::insert_into(membership_events::table)
            .values(self)
            .execute(conn)?;
        Ok(())
    }
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
    pub fn new(name: &str, creator: &User) -> Self {
        NewWorkspace {
            name: name.to_string(),
            creator_id: creator.id.clone(),
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

            NewMembershipEvent::new(
                &workspace_id,
                &self.creator_id,
                &self.creator_id,
                "owner_assigned",
                Some(Role::Owner.as_str()),
            )
            .insert(conn)?;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Viewer,
    Editor,
    Owner,
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Role {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let level = |r: &Role| match r {
            Role::Viewer => 0,
            Role::Editor => 1,
            Role::Owner => 2,
        };
        level(self).cmp(&level(other))
    }
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

    fn make_user(conn: &mut SqliteConnection, email: &str) -> User {
        NewUser::new(email, "Test User", "password")
            .expect("new user should succeed")
            .insert(conn)
            .expect("user insert should succeed")
    }

    fn make_workspace(conn: &mut SqliteConnection, name: &str, creator: &User) -> Workspace {
        NewWorkspace::new(name, creator)
            .insert(conn)
            .expect("workspace creation should succeed")
    }

    #[test]
    fn create_workspace_succeeds() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user);
        assert_eq!(ws.name, "My Finances");
        assert!(!ws.id.is_empty());
    }

    #[test]
    fn creator_is_assigned_owner() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user);

        let role = ws.user_role(&user, &mut conn)
            .expect("role query should succeed")
            .expect("creator should have a role");

        assert_eq!(role, Role::Owner);
    }

    #[test]
    fn user_can_belong_to_multiple_workspaces() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        make_workspace(&mut conn, "Workspace A", &user);
        make_workspace(&mut conn, "Workspace B", &user);

        let workspaces = Workspace::for_user(&user, &mut conn)
            .expect("for_user should succeed");
        assert_eq!(workspaces.len(), 2);
    }

    #[test]
    fn owner_event_is_appended_to_log() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user);

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
        let user = make_user(&mut conn, "alice@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &user);

        let err = ws.remove_member(&user, &user, &mut conn)
            .expect_err("removing last owner should fail");

        assert!(matches!(err, AppError::Conflict(_)));

        let count = ws.num_owners(&mut conn)
            .expect("num_owners should succeed");
        assert_eq!(count, 1, "workspace must still have one owner");
    }

    #[test]
    fn non_member_has_no_role() {
        let mut conn = setup_test_db();
        let owner = make_user(&mut conn, "owner@example.com");
        let outsider = make_user(&mut conn, "outsider@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &owner);

        let role = ws.user_role(&outsider, &mut conn)
            .expect("role query should succeed");
        assert!(role.is_none(), "outsider should have no role");
    }

    #[test]
    fn is_member_returns_false_for_non_member() {
        let mut conn = setup_test_db();
        let owner = make_user(&mut conn, "owner@example.com");
        let outsider = make_user(&mut conn, "outsider@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &owner);

        assert!(!ws.is_member(&outsider, &mut conn)
            .expect("is_member should succeed"));
        assert!(ws.is_member(&owner, &mut conn)
            .expect("is_member should succeed"));
    }

    #[test]
    fn workspace_data_is_scoped_by_workspace_id() {
        let mut conn = setup_test_db();
        let user_a = make_user(&mut conn, "a@example.com");
        let user_b = make_user(&mut conn, "b@example.com");
        let ws_a = make_workspace(&mut conn, "Workspace A", &user_a);
        let ws_b = make_workspace(&mut conn, "Workspace B", &user_b);

        let members_b = ws_b.members(&mut conn)
            .expect("members query should succeed");
        assert!(
            members_b.iter().all(|m| m.user_id != user_a.id),
            "user_a must not appear in workspace B"
        );

        let members_a = ws_a.members(&mut conn)
            .expect("members query should succeed");
        assert!(
            members_a.iter().all(|m| m.user_id != user_b.id),
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

        let a_workspaces = Workspace::for_user(&user_a, &mut conn)
            .expect("for_user should succeed");

        assert_eq!(a_workspaces.len(), 1);
        assert_eq!(a_workspaces[0].name, "A's Workspace");
    }

    #[test]
    fn non_owner_can_be_removed() {
        let mut conn = setup_test_db();
        let owner = make_user(&mut conn, "owner@example.com");
        let editor = make_user(&mut conn, "editor@example.com");
        let ws = make_workspace(&mut conn, "My Finances", &owner);

        diesel::insert_into(workspace_members::table)
            .values(NewWorkspaceMember {
                workspace_id: ws.id.clone(),
                user_id: editor.id.clone(),
                role: Role::Editor.as_str().to_string(),
                joined_at: chrono::Utc::now().naive_utc(),
            })
            .execute(&mut conn)
            .expect("insert editor should succeed");

        ws.remove_member(&editor, &owner, &mut conn)
            .expect("removing editor should succeed");

        let role = ws.user_role(&editor, &mut conn)
            .expect("role query should succeed");
        assert!(role.is_none(), "editor should no longer be a member");
    }
}
