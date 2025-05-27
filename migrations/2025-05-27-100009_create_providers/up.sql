CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE providers (
    id UUID PRIMARY KEY,     
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    url TEXT NOT NULL,
    is_active BOOL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()   
);