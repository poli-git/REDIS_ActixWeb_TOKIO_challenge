CREATE TABLE events (
    id UUID PRIMARY KEY,
    providers_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    is_active BOOL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),


    foreign key (providers_id) references providers(id)
);