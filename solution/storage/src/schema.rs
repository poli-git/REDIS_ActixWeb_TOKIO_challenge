// @generated automatically by Diesel CLI.

diesel::table! {
    base_plans (base_plans_id) {
        base_plans_id -> Uuid,
        providers_id -> Uuid,
        event_base_id -> Text,
        title -> Text,
        sell_mode -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    plans (plans_id) {
        plans_id -> Uuid,
        base_plans_id -> Uuid,
        event_plan_id -> Text,
        plan_start_date -> Timestamp,
        plan_end_date -> Timestamp,
        sell_from -> Timestamp,
        sell_to -> Timestamp,
        sold_out -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    providers (providers_id) {
        providers_id -> Uuid,
        name -> Text,
        description -> Text,
        url -> Text,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    zones (zones_id) {
        zones_id -> Uuid,
        plans_id -> Uuid,
        event_zone_id -> Text,
        name -> Text,
        numbered -> Bool,
        capacity -> Text,
        price -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(base_plans -> providers (providers_id));
diesel::joinable!(plans -> base_plans (base_plans_id));
diesel::joinable!(zones -> plans (plans_id));

diesel::allow_tables_to_appear_in_same_query!(base_plans, plans, providers, zones,);
