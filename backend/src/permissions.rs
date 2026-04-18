use crate::domain::user::UserRef;
use crate::domain::workspace::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    WorkspaceSelf,
    WorkspaceMember,
    WorkspaceOwner,
}

impl Role {
    pub fn has_permission(&self, action: Action, resource: Resource) -> bool {
        matches!(
            (self, action, resource),
            // Anyone can remove themselves
            (_, Action::Delete, Resource::WorkspaceSelf) |
            // Owners can add and remove members
            (Role::Owner, Action::Create, Resource::WorkspaceMember) |
            (Role::Owner, Action::Delete, Resource::WorkspaceMember)
        )
    }
}

/// Determine the resource type for a membership action.
pub fn membership_resource(actor: &UserRef, target: &UserRef, target_role: &Role) -> Resource {
    if actor.id == target.id {
        Resource::WorkspaceSelf
    } else if *target_role >= Role::Owner {
        Resource::WorkspaceOwner
    } else {
        Resource::WorkspaceMember
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str) -> UserRef {
        UserRef::new(id)
    }

    // ── has_permission ───────────────────────────────────────────────────

    #[test]
    fn any_role_can_delete_self() {
        for role in [Role::Viewer, Role::Editor, Role::Owner] {
            assert!(role.has_permission(Action::Delete, Resource::WorkspaceSelf));
        }
    }

    #[test]
    fn owner_can_delete_member() {
        assert!(Role::Owner.has_permission(Action::Delete, Resource::WorkspaceMember));
    }

    #[test]
    fn owner_cannot_delete_owner() {
        assert!(!Role::Owner.has_permission(Action::Delete, Resource::WorkspaceOwner));
    }

    #[test]
    fn editor_cannot_delete_member() {
        assert!(!Role::Editor.has_permission(Action::Delete, Resource::WorkspaceMember));
    }

    #[test]
    fn viewer_cannot_delete_member() {
        assert!(!Role::Viewer.has_permission(Action::Delete, Resource::WorkspaceMember));
    }

    #[test]
    fn owner_can_create_member() {
        assert!(Role::Owner.has_permission(Action::Create, Resource::WorkspaceMember));
    }

    #[test]
    fn editor_cannot_create_member() {
        assert!(!Role::Editor.has_permission(Action::Create, Resource::WorkspaceMember));
    }

    // ── membership_resource ──────────────────────────────────────────────

    #[test]
    fn self_action_returns_workspace_self() {
        assert_eq!(membership_resource(&user("u1"), &user("u1"), &Role::Owner), Resource::WorkspaceSelf);
    }

    #[test]
    fn targeting_owner_returns_workspace_owner() {
        assert_eq!(membership_resource(&user("u1"), &user("u2"), &Role::Owner), Resource::WorkspaceOwner);
    }

    #[test]
    fn targeting_editor_returns_workspace_member() {
        assert_eq!(membership_resource(&user("u1"), &user("u2"), &Role::Editor), Resource::WorkspaceMember);
    }

    #[test]
    fn targeting_viewer_returns_workspace_member() {
        assert_eq!(membership_resource(&user("u1"), &user("u2"), &Role::Viewer), Resource::WorkspaceMember);
    }
}
