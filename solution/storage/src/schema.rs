// @generated automatically by Diesel CLI.

diesel::table! {
    event (id) {
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
    provider (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        url -> Text,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(event -> provider (providers_id));

diesel::allow_tables_to_appear_in_same_query!(
    event,
    provider,
);
