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
        role -> Nullable<Text>,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    entries (id) {
        id -> Text,
        workspace_id -> Text,
        date -> Date,
        description -> Text,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    postings (id) {
        id -> Text,
        entry_id -> Text,
        account_id -> Text,
        amount -> Integer,
        note -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    baselines (id) {
        id -> Text,
        workspace_id -> Text,
        account_id -> Text,
        period_type -> Text,
        period_start -> Date,
        amount -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    auth_sessions (id) {
        id -> Text,
        data -> Binary,
        expiry_date -> BigInt,
    }
}

diesel::table! {
    recurring_rules (id) {
        id -> Text,
        workspace_id -> Text,
        description -> Text,
        rrule -> Text,
        dtstart -> Date,
        last_fired_on -> Nullable<Date>,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    recurring_posting_templates (id) {
        id -> Text,
        rule_id -> Text,
        account_id -> Text,
        amount -> Integer,
        note -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::joinable!(accounts -> workspaces (workspace_id));
diesel::joinable!(entries -> workspaces (workspace_id));
diesel::joinable!(postings -> entries (entry_id));
diesel::joinable!(postings -> accounts (account_id));
diesel::joinable!(baselines -> workspaces (workspace_id));
diesel::joinable!(baselines -> accounts (account_id));
diesel::joinable!(recurring_rules -> workspaces (workspace_id));
diesel::joinable!(recurring_posting_templates -> recurring_rules (rule_id));
diesel::joinable!(recurring_posting_templates -> accounts (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    baselines,
    workspace_members,
    workspaces,
    accounts,
    entries,
    postings,
    auth_sessions,
    users,
    recurring_rules,
    recurring_posting_templates,
);
