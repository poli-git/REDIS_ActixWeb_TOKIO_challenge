CREATE TABLE zones (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id uuid NOT NULL,
    zone_id bigint NOT NULL,
    name TEXT NOT NULL,
    numbered BOOL NOT NULL DEFAULT FALSE,
    capacity bigint NOT NULL,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (plan_id) references plans(id)
    
);