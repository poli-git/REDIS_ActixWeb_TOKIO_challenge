CREATE TABLE providers (
    id uuid PRIMARY KEY,
    providers_id bigint NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    url TEXT NOT NULL,
    is_active BOOL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()

);