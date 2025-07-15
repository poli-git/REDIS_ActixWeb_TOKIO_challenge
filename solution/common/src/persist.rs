use crate::utils::{get_cache, get_db_connection};
use crate::xml_models;
use crate::xml_models::{EventOutput, SellModeEnum};

use storage::base_plan::add_or_update_base_plan;
use storage::connections::db::PgPooledConnection;
use storage::models::base_plans::NewBasePlan;
use storage::models::plans::NewPlan;
use storage::models::zones::NewZone;
use storage::plan::add_or_update_plan;
use storage::zone::add_or_update_zone;

// Import or define PersistPlansError
use crate::error::PersistPlansError;

// Import serde_json for serialization
use serde_json;

pub async fn persist_base_plans(
    base_plans: Vec<xml_models::BasePlan>,
    provider_id: uuid::Uuid,
    provider_name: String,
    expiration_key_time_limit: u32,
) -> Result<(), PersistPlansError> {
    if base_plans.is_empty() {
        log::warn!(
            "No base plans found for provider: {} - {}",
            provider_id,
            provider_name
        );
        return Ok(());
    }
    // Get DB connection
    let mut pg_pool = match get_db_connection().await {
        Some(conn) => conn,
        None => {
            return Err(PersistPlansError::DbError(
                "Failed to get DB connection".to_string(),
            ));
        }
    };

    for bp in base_plans {
        let new_base_plan = NewBasePlan {
            base_plans_id: uuid::Uuid::new_v4(),
            providers_id: provider_id,
            event_base_id: bp.base_plan_id.clone().unwrap_or_default(),
            title: bp.title.clone(),
            sell_mode: bp
                .sell_mode
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default(),
        };

        // Persist the base_plan to the database and cache
        match add_or_update_base_plan(&mut pg_pool, new_base_plan) {
            Ok(inserted) => {
                log::debug!(
                    "Added base_plan: {} : {}",
                    inserted.event_base_id,
                    inserted.title
                );

                // Persist plans associated with this base plan and cache
                match persist_plans(
                    &bp.plans,
                    bp.sell_mode.as_ref().cloned(),
                    inserted.base_plans_id,
                    &inserted.event_base_id,
                    inserted.providers_id,
                    &inserted.title,
                    &mut pg_pool,
                    expiration_key_time_limit,
                )
                .await
                {
                    Ok(_) => {
                        log::debug!(
                            "Successfully persisted plans for base plan ID: {}",
                            inserted.base_plans_id
                        );
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to persist plans for base plan ID {}: {}",
                            inserted.base_plans_id,
                            e
                        );
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to add base_plan: {}", e);
                return Err(PersistPlansError::DbError(e.to_string()));
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn persist_plans(
    bp_plans: &Vec<xml_models::Plan>,
    sell_mode: Option<SellModeEnum>,
    base_plans_id: uuid::Uuid,
    event_base_id: &str,
    provider_id: uuid::Uuid,
    title: &str,
    pg_pool: &mut PgPooledConnection,
    expiration_key_time_limit: u32,
) -> Result<(), PersistPlansError> {
    if bp_plans.is_empty() {
        log::warn!("No plans found for base_plan ID: {}", base_plans_id);
        return Err(PersistPlansError::NotFound(format!(
            "No plans found for base_plan ID: {}",
            base_plans_id
        )));
    }
    log::debug!(
        "Persisting {} plans for base_plan ID: {}",
        bp_plans.len(),
        base_plans_id
    );
    // Clone sell_mode so it can be used multiple times
    let sell_mode_clone = sell_mode.clone();

    for plan in bp_plans {
        let new_plan = NewPlan {
            plans_id: uuid::Uuid::new_v4(),
            base_plans_id,
            event_plan_id: plan.plan_id.clone().unwrap_or_default(),
            plan_start_date: chrono::NaiveDateTime::parse_from_str(
                &plan.plan_start_date,
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            plan_end_date: chrono::NaiveDateTime::parse_from_str(
                &plan.plan_end_date,
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            sell_from: plan
                .sell_from
                .as_ref()
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok())
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            sell_to: plan
                .sell_to
                .as_ref()
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok())
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            sold_out: plan.sold_out.unwrap_or(false),
        };

        match add_or_update_plan(pg_pool, new_plan) {
            Ok(inserted_plan) => {
                log::debug!(
                    "Added plan: {} : {}",
                    inserted_plan.plans_id,
                    inserted_plan.event_plan_id
                );
                // Cache ONLY plan dates that are associated to a base_plan with sell mode = 'online'
                if sell_mode_clone.as_ref() == Some(&SellModeEnum::Online) {
                    // Get Cache instance
                    let redis_conn = get_cache().await;
                    if let Err(e) = redis_conn
                        .cache_plan_dates(
                            event_base_id.to_string(),
                            inserted_plan.event_plan_id.to_string(),
                            inserted_plan.plan_start_date,
                            inserted_plan.plan_end_date,
                        )
                        .await
                    {
                        log::error!(
                            "Failed to cache start/end date for online event {}: {}",
                            inserted_plan.event_plan_id.to_string(),
                            e
                        );
                        return Err(PersistPlansError::RedisError(e.to_string()));
                    }

                    let new_event = EventOutput {
                        base_plan_id: Some(event_base_id.to_string()),
                        title: Some(title.to_owned()),
                        sell_mode: Some(sell_mode_clone.clone().unwrap_or(SellModeEnum::Online)),
                        plan: plan.clone(),
                    };
                    // Cache the online plan
                    if let Err(e) = redis_conn
                        .set_ex(
                            format!(
                                "plan:{}:{}:{}",
                                provider_id, event_base_id, inserted_plan.event_plan_id
                            )
                            .as_str(),
                            serde_json::to_string(&new_event).unwrap_or_default(),
                            expiration_key_time_limit.try_into().unwrap_or(60 * 60 * 24),
                        )
                        .await
                    {
                        log::error!(
                            "Failed to cache online base_plan {}:{}:{}: {}",
                            provider_id,
                            event_base_id,
                            inserted_plan.event_plan_id,
                            e
                        );
                    }
                }
                // Convert xml_models::Zone to NewZone before persisting
                let new_zones: Vec<NewZone> = plan
                    .zones
                    .iter()
                    .map(|zone| NewZone {
                        zones_id: uuid::Uuid::new_v4(),
                        plans_id: inserted_plan.plans_id,
                        name: zone.name.clone().unwrap_or_default(),
                        capacity: zone.capacity.clone().unwrap_or_default(),
                        event_zone_id: zone.zone_id.clone().unwrap_or_default(),
                        price: zone.price.clone().unwrap_or_default(),
                        numbered: zone.numbered.unwrap_or_default(),
                        // Add other fields as required by NewZone struct
                    })
                    .collect();

                // Persist zones for the plan
                log::debug!(
                    "Persisting {} zones for plan ID: {}",
                    new_zones.len(),
                    inserted_plan.plans_id
                );
                if let Err(e) = persist_zones(&new_zones, pg_pool).await {
                    log::error!(
                        "Failed to persist zones for plan {}: {}",
                        inserted_plan.plans_id,
                        e
                    );
                    return Err(e);
                }
            }
            Err(e) => {
                log::error!("Failed to add plan: {}", e);
                return Err(PersistPlansError::DbError(e.to_string()));
            }
        }
    }
    Ok(())
}

async fn persist_zones(
    zones: &Vec<NewZone>,
    pg_pool: &mut PgPooledConnection,
) -> Result<(), PersistPlansError> {
    if zones.is_empty() {
        log::warn!("No zones to persist for this plan.");
        return Ok(());
    }
    for zone in zones {
        match add_or_update_zone(pg_pool, zone.clone()) {
            Ok(inserted_zone) => {
                log::debug!(
                    "Added/updated zone: {} for plan: {}",
                    inserted_zone.zones_id,
                    inserted_zone.plans_id
                );
            }
            Err(e) => {
                log::error!(
                    "Failed to add/update zone {} for plan {}: {}",
                    zone.zones_id,
                    zone.plans_id,
                    e
                );
                return Err(PersistPlansError::DbError(e.to_string()));
            }
        }
    }
    Ok(())
}
