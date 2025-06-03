ALTER TABLE events
ADD CONSTRAINT constraint_events_provider UNIQUE (id, providers_id);