use chrono::NaiveDateTime;
use diesel::deserialize::{self, FromSql};
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel_derive_newtype::DieselNewType;
use uuid::Uuid;

use crate::domain::user::UserRef;
use crate::error::AppError;
use crate::schema::{membership_events, workspace_members, workspaces};

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, DieselNewType)]
#[serde(transparent)]
pub struct WorkspaceRef(String);

impl std::fmt::Display for WorkspaceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Queryable, Selectable, serde::Serialize)]
#[diesel(table_name = workspaces)]
pub struct Workspace {
    #[serde(rename = "workspace")]
    pub id: WorkspaceRef,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Workspace {
    pub fn as_ref(&self) -> &WorkspaceRef {
        &self.id
    }

    pub fn from_ref(
        workspace_ref: &WorkspaceRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<Workspace>, AppError> {
        workspaces::table
            .find(workspace_ref)
            .select(Workspace::as_select())
            .first(conn)
            .optional()
            .map_err(|err| AppError::Internal(format!("workspace lookup failed: {err}")))
    }

    pub fn view_user(
        &self,
        user: &UserRef,
        conn: &mut SqliteConnection,
    ) -> Result<Option<WorkspaceUser>, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .filter(workspace_members::user_id.eq(user))
            .select(WorkspaceUser::as_select())
            .first(conn)
            .optional()
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

    pub fn members(&self, conn: &mut SqliteConnection) -> Result<Vec<WorkspaceUser>, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .select(WorkspaceUser::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("members query failed: {err}")))
    }

    pub fn is_member(&self, user: &UserRef, conn: &mut SqliteConnection) -> Result<bool, AppError> {
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(&self.id))
            .filter(workspace_members::user_id.eq(user))
            .count()
            .get_result::<i64>(conn)
            .map(|count| count > 0)
            .map_err(|err| AppError::Internal(format!("is_member query failed: {err}")))
    }

    pub fn remove_member(
        &self,
        user: &UserRef,
        actor: &UserRef,
        conn: &mut SqliteConnection,
    ) -> Result<(), AppError> {
        conn.transaction::<_, AppError, _>(|conn| {
            let role = workspace_members::table
                .filter(workspace_members::workspace_id.eq(&self.id))
                .filter(workspace_members::user_id.eq(&user))
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

            let event_type = if user == actor { "left" } else { "removed" };

            NewMembershipEvent::new(&self.id, &user, &actor, event_type, None).insert(conn)?;

            diesel::delete(
                workspace_members::table
                    .filter(workspace_members::workspace_id.eq(&self.id))
                    .filter(workspace_members::user_id.eq(&user)),
            )
            .execute(conn)?;

            Ok(())
        })
    }
}

impl UserRef {
    pub fn workspaces(&self, conn: &mut SqliteConnection) -> Result<Vec<Workspace>, AppError> {
        workspace_members::table
            .filter(workspace_members::user_id.eq(self))
            .inner_join(workspaces::table.on(workspaces::id.eq(workspace_members::workspace_id)))
            .select(Workspace::as_select())
            .load(conn)
            .map_err(|err| AppError::Internal(format!("find workspaces for user failed: {err}")))
    }
}

#[derive(Debug, Queryable, Selectable, serde::Serialize)]
#[diesel(table_name = workspace_members)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WorkspaceUser {
    #[diesel(column_name = workspace_id)]
    #[serde(skip_serializing)]
    pub workspace: WorkspaceRef,
    #[diesel(column_name = user_id)]
    pub user: UserRef,
    pub role: Role,
    pub joined_at: NaiveDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = membership_events)]
pub struct MembershipEvent {
    pub id: String,
    #[diesel(column_name = workspace_id)]
    pub workspace: String,
    #[diesel(column_name = user_id)]
    pub user: String,
    #[diesel(column_name = actor_id)]
    pub actor: String,
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
    #[diesel(column_name = workspace_id)]
    workspace: WorkspaceRef,
    #[diesel(column_name = user_id)]
    user: UserRef,
    #[diesel(column_name = actor_id)]
    actor: UserRef,
    event_type: String,
    role: Option<String>,
    created_at: NaiveDateTime,
}

impl NewMembershipEvent {
    fn new(
        workspace: &WorkspaceRef,
        user: &UserRef,
        actor: &UserRef,
        event_type: &str,
        role: Option<&str>,
    ) -> Self {
        NewMembershipEvent {
            id: Uuid::new_v4().to_string(),
            workspace: workspace.clone(),
            user: user.clone(),
            actor: actor.clone(),
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
struct NewWorkspaceUser {
    #[diesel(column_name = workspace_id)]
    workspace: String,
    #[diesel(column_name = user_id)]
    user: UserRef,
    role: String,
    joined_at: NaiveDateTime,
}

// ── NewWorkspace ────────────────────────────────────────────────────────────

pub struct NewWorkspace {
    name: String,
    creator: UserRef,
}

impl NewWorkspace {
    pub fn new(name: &str, creator: UserRef) -> Self {
        NewWorkspace {
            name: name.to_string(),
            creator,
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
                &workspace.as_ref(),
                &self.creator,
                &self.creator,
                "owner_assigned",
                Some(Role::Owner.as_str()),
            )
            .insert(conn)?;

            diesel::insert_into(workspace_members::table)
                .values(NewWorkspaceUser {
                    workspace: workspace_id,
                    user: self.creator,
                    role: Role::Owner.as_str().to_string(),
                    joined_at: now,
                })
                .execute(conn)?;

            Ok(workspace)
        })
    }
}

// ── Role ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

impl FromSql<Text, Sqlite> for Role {
    fn from_sql(
        bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Role::from_str(&s).ok_or_else(|| format!("unknown role: {s}").into())
    }
}

impl ToSql<Text, Sqlite> for Role {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(self.as_str(), out)
    }
}

// Required for Queryable derive to work with Role as a field
impl diesel::deserialize::Queryable<Text, Sqlite> for Role {
    type Row = Self;
    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(row)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::{NewUser, User};
    use crate::test_helpers::setup_test_db;

    fn make_user(conn: &mut SqliteConnection, email: &str) -> User {
        NewUser::new(email, "Test User", "password")
            .expect("new user should succeed")
            .insert(conn)
            .expect("user insert should succeed")
    }

    fn make_workspace(conn: &mut SqliteConnection, name: &str, creator: &User) -> Workspace {
        NewWorkspace::new(name, creator.as_ref().clone())
            .insert(conn)
            .expect("workspace creation should succeed")
    }

    #[test]
    fn create_workspace_succeeds() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        let workspace = make_workspace(&mut conn, "My Finances", &user);
        assert_eq!(workspace.name, "My Finances");
    }

    #[test]
    fn creator_is_assigned_owner() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        let workspace = make_workspace(&mut conn, "My Finances", &user);

        let member = workspace
            .view_user(user.as_ref(), &mut conn)
            .expect("view_user should succeed")
            .expect("creator should be a member");

        assert_eq!(member.role, Role::Owner);
    }

    #[test]
    fn user_can_belong_to_multiple_workspaces() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        make_workspace(&mut conn, "Workspace A", &user);
        make_workspace(&mut conn, "Workspace B", &user);

        let workspaces = user
            .as_ref()
            .workspaces(&mut conn)
            .expect("workspaces should succeed");
        assert_eq!(workspaces.len(), 2);
    }

    #[test]
    fn owner_event_is_appended_to_log() {
        let mut conn = setup_test_db();
        let user = make_user(&mut conn, "alice@example.com");
        let workspace = make_workspace(&mut conn, "My Finances", &user);

        let events: Vec<MembershipEvent> = membership_events::table
            .filter(membership_events::workspace_id.eq(&workspace.id))
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
        let workspace = make_workspace(&mut conn, "My Finances", &user);

        let err = workspace
            .remove_member(user.as_ref(), user.as_ref(), &mut conn)
            .expect_err("removing last owner should fail");

        assert!(matches!(err, AppError::Conflict(_)));

        let count = workspace
            .num_owners(&mut conn)
            .expect("num_owners should succeed");
        assert_eq!(count, 1, "workspace must still have one owner");
    }

    #[test]
    fn non_member_has_no_role() {
        let mut conn = setup_test_db();
        let owner = make_user(&mut conn, "owner@example.com");
        let outsider = make_user(&mut conn, "outsider@example.com");
        let workspace = make_workspace(&mut conn, "My Finances", &owner);

        let member = workspace
            .view_user(outsider.as_ref(), &mut conn)
            .expect("view_user should succeed");
        assert!(member.is_none(), "outsider should have no role");
    }

    #[test]
    fn is_member_returns_false_for_non_member() {
        let mut conn = setup_test_db();
        let owner = make_user(&mut conn, "owner@example.com");
        let outsider = make_user(&mut conn, "outsider@example.com");
        let workspace = make_workspace(&mut conn, "My Finances", &owner);

        assert!(!workspace
            .is_member(outsider.as_ref(), &mut conn)
            .expect("is_member should succeed"));
        assert!(workspace
            .is_member(owner.as_ref(), &mut conn)
            .expect("is_member should succeed"));
    }

    #[test]
    fn workspace_data_is_scoped_by_workspace_id() {
        let mut conn = setup_test_db();
        let user_a = make_user(&mut conn, "a@example.com");
        let user_b = make_user(&mut conn, "b@example.com");
        let workspace_a = make_workspace(&mut conn, "Workspace A", &user_a);
        let workspace_b = make_workspace(&mut conn, "Workspace B", &user_b);

        let members_b = workspace_b
            .members(&mut conn)
            .expect("members query should succeed");
        assert!(
            members_b.iter().all(|m| &m.user != user_a.as_ref()),
            "user_a must not appear in workspace B"
        );

        let members_a = workspace_a
            .members(&mut conn)
            .expect("members query should succeed");
        assert!(
            members_a.iter().all(|m| &m.user != user_b.as_ref()),
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

        let a_workspaces = user_a
            .as_ref()
            .workspaces(&mut conn)
            .expect("workspaces should succeed");

        assert_eq!(a_workspaces.len(), 1);
        assert_eq!(a_workspaces[0].name, "A's Workspace");
    }

    #[test]
    fn non_owner_can_be_removed() {
        let mut conn = setup_test_db();
        let owner = make_user(&mut conn, "owner@example.com");
        let editor = make_user(&mut conn, "editor@example.com");
        let workspace = make_workspace(&mut conn, "My Finances", &owner);

        diesel::insert_into(workspace_members::table)
            .values(NewWorkspaceUser {
                workspace: workspace.id.to_string(),
                user: editor.as_ref().clone(),
                role: Role::Editor.as_str().to_string(),
                joined_at: chrono::Utc::now().naive_utc(),
            })
            .execute(&mut conn)
            .expect("insert editor should succeed");

        workspace
            .remove_member(editor.as_ref(), owner.as_ref(), &mut conn)
            .expect("removing editor should succeed");

        let member = workspace
            .view_user(editor.as_ref(), &mut conn)
            .expect("view_user should succeed");
        assert!(member.is_none(), "editor should no longer be a member");
    }
}
