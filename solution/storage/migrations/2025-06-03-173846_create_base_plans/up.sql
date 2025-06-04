CREATE TABLE base_plans (
    base_plans_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    providers_id uuid NOT NULL,
    event_base_id bigint NOT NULL,
    title TEXT NOT NULL,
    sell_mode TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (providers_id) references providers(providers_id),
    UNIQUE (providers_id, event_base_id)
);

