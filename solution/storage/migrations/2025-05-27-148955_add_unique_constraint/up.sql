DROP CONSTRAINT IF EXISTS constraint_providers_id;
ALTER TABLE providers
ADD CONSTRAINT constraint_providers_id UNIQUE (providers_id);

DROP CONSTRAINT IF EXISTS constraint_base_plans_provider;   
ALTER TABLE base_plans
ADD CONSTRAINT constraint_base_plans_provider UNIQUE (base_plan_id, providers_id);

DROP CONSTRAINT IF EXISTS constraint_plans_base_plan;
ALTER TABLE plans
ADD CONSTRAINT constraint_plans_base_plan UNIQUE (plan_id, base_plan_id);

DROP CONSTRAINT IF EXISTS constraint_zones_plan;
ALTER TABLE zones
ADD CONSTRAINT constraint_zones_plan UNIQUE (zone_id, plan_id);