// @generated automatically by Diesel CLI.

diesel::table! {
    events (id) {
        id -> Uuid,
        providers_id -> Uuid,
        name -> Text,
        description -> Text,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    providers (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        url -> Text,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(events -> providers (providers_id));

diesel::allow_tables_to_appear_in_same_query!(
    events,
    providers,
);
