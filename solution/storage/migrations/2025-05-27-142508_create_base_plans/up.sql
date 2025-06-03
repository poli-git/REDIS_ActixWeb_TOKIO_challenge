CREATE TABLE base_plans (
    id uuid PRIMARY KEY,
    base_plan_id bigint,
    providers_id bigint NOT NULL,
    title TEXT NOT NULL,
    sell_mode TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (providers_id) references providers(providers_id)
);