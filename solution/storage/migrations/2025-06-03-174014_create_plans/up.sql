CREATE TABLE plans (
    plans_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    base_plans_id uuid NOT NULL,    
    event_plan_id bigint NOT NULL,
    plan_start_date TIMESTAMP NOT NULL,
    plan_end_date TIMESTAMP NOT NULL,
    sell_from TIMESTAMP NOT NULL,
    sell_to TIMESTAMP NOT NULL,
    sold_out BOOL NOT NULL DEFAULT FALSE,    
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),


    FOREIGN KEY (base_plans_id) references base_plans(base_plans_id)
    
  
);
