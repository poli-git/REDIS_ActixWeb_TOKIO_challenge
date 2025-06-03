// @generated automatically by Diesel CLI.

diesel::table! {
    base_plans (id) {
        id -> Uuid,
        providers_id -> Uuid,
        base_plan_id -> Int8,
        title -> Text,
        sell_mode -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    plans (id) {
        id -> Uuid,
        base_plan_id -> Uuid,
        plan_id -> Int8,
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
    zones (id) {
        id -> Uuid,
        plan_id -> Uuid,
        zone_id -> Int8,
        name -> Text,
        numbered -> Bool,
        capacity -> Int8,
        price -> Float8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(base_plans -> providers (providers_id));
diesel::joinable!(plans -> base_plans (base_plan_id));
diesel::joinable!(zones -> plans (plan_id));

diesel::allow_tables_to_appear_in_same_query!(base_plans, plans, providers, zones,);
