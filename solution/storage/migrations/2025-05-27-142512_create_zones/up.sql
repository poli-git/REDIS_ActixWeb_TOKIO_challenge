CREATE TABLE zones (
    id uuid PRIMARY KEY,
    zone_id bigint NOT NULL,
    plan_id bigint NOT NULL,
    name TEXT NOT NULL,
    numbered BOOL NOT NULL DEFAULT FALSE,
    capacity bigint NOT NULL,
    price double NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (plan_id) references plans(plan_id)
);