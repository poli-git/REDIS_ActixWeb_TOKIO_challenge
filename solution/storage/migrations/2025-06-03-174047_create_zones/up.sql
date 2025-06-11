CREATE TABLE zones (
    zones_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    plans_id uuid NOT NULL,
    event_zone_id TEXT NOT NULL,
    name TEXT NOT NULL,
    numbered BOOL NOT NULL DEFAULT FALSE,
    capacity TEXT NOT NULL,
    price TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (plans_id) references plans(plans_id),
    UNIQUE (plans_id, event_zone_id, numbered)    
    
);