diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        display_name -> Text,
        password_hash -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Text,
        name -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    membership_events (id) {
        id -> Text,
        workspace_id -> Text,
        user_id -> Text,
        actor_id -> Text,
        event_type -> Text,
        role -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    workspace_members (workspace_id, user_id) {
        workspace_id -> Text,
        user_id -> Text,
        role -> Text,
        joined_at -> Text,
    }
}

diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::allow_tables_to_appear_in_same_query!(workspace_members, workspaces);
