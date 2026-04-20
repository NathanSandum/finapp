diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        display_name -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Text,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        created_at -> Timestamp,
    }
}

diesel::table! {
    workspace_members (workspace_id, user_id) {
        workspace_id -> Text,
        user_id -> Text,
        role -> Text,
        joined_at -> Timestamp,
    }
}

diesel::table! {
    accounts (id) {
        id -> Text,
        workspace_id -> Text,
        parent_id -> Nullable<Text>,
        name -> Text,
        account_type -> Text,
        currency -> Text,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions (id) {
        id -> Text,
        workspace_id -> Text,
        date -> Text,
        description -> Text,
        cleared -> Bool,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        user_id -> Text,
        token_hash -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(accounts -> workspaces (workspace_id));
diesel::joinable!(transactions -> workspaces (workspace_id));

diesel::allow_tables_to_appear_in_same_query!(
    workspace_members,
    workspaces,
    accounts,
    transactions,
    sessions,
    users,
);
