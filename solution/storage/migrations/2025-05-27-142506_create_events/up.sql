CREATE TABLE events (
    id uuid PRIMARY KEY,
    providers_id uuid NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    is_active BOOL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    foreign key (providers_id) references providers(id)
);