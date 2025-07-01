use chrono::NaiveDateTime;
use serde::Serialize;
use std::collections::HashMap;
use storage::connections::cache::{Cache, ProviderABaseEvent};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub data: T,
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EventsData {
    pub events: Vec<EventDTO>,
}

#[derive(Serialize, ToSchema)]
pub struct EventDTO {
    pub id: String,
    pub title: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub min_price: f64,
    pub max_price: f64,
}

pub async fn get_plans(
    cache: &Cache,
    starts_at: NaiveDateTime,
    ends_at: NaiveDateTime,
) -> Result<ApiResponse<EventsData>, String> {
    // Fetch matched plans from the cache
    let matched_event_ids = match cache.get_plans(starts_at, ends_at).await {
        Ok(matched_event_ids) => matched_event_ids,
        Err(e) => return Err(format!("Failed to fetch plans: {}", e)),
    };

    for event_id in matched_event_ids {
        let parts: Vec<&str> = event_id.split(':').collect();
        if parts.len() != 2 {
            error!("Invalid event ID format: {}", event_id);
            return Err(format!("Invalid event ID format: {}", event_id));
        }
        let base_id = parts[0].to_string();
        let plan_id = parts[1].to_string();
        let base_id_clone = base_id.clone();

        let key = format!("{}:{}:{}:{}", ROOT_KEY, "*", base_id, plan_id);

        let scan_result = match cache.get_keys_matching_pattern(&key) {
            Ok(scan_result) => scan_result,
            Err(e) => {
                error!("Error scanning Redis for key pattern {}: {}", key, e);
                return Err(format!("Redis scan error: {}", e));
            }
        };
        if scan_result.is_empty() {
            error!("No plans found for base ID: {}", base_id);
            return Err(format!("Redis scan error: {}", e));
        }

        // Get plans stored in Redis for the given base_id and plan_id
        // Iterate over the results and deserialize each plan
        for result in scan_result {
            if result.trim().is_empty() {
                error!("Plan string from Redis is empty for key: {}", key);
                continue;
            }
            let result_clone = result.clone();
            let plan = self.get(result).await.map_err(|e| {
                error!("Error getting plan from Redis: {}", e);
                CacheError::Error(format!("Redis get error: {}", e))
            })?;
            if plan.trim().is_empty() {
                error!("Plan string is empty for key: {}", result_clone);
                continue;
            }

            ;

            let plan: ProviderABaseEvent = serde_json::from_str(&plan_json).map_err(|e| {
                error!("Error deserializing plan: {} | raw value: {}", e, plan_json);
                CacheError::Error(format!("Deserialization error: {}", e))
            })?;

            // Insert the plan into the base_events map
            base_events
                .entry(base_id_clone.clone())
                .or_insert_with(Vec::new)
                .push(plan);
        }
    }
}

pub fn map_provider_events_to_response_dto(
    base_events: &[ProviderABaseEvent],
) -> ApiResponse<EventsData> {
    let mut events_data = Vec::new();

    if base_events.is_empty() {
        return ApiResponse {
            data: EventsData {
                events: events_data,
            },
            error: None,
        };
    }
    // Iterate over each base event and extract relevant data
    for base_event in base_events {
        let plan = &base_event.plan;

        // Parse start and end datetime
        let (start_date, start_time) = split_datetime(&plan.plan_start_date);
        let (end_date, end_time) = split_datetime(&plan.plan_end_date);

        // Compute min and max price
        let prices: Vec<f64> = plan
            .zones
            .iter()
            .filter_map(|z| z.price.parse::<f64>().ok())
            .collect();

        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Create EventDTO and push to the events_data vector
        events_data.push(EventDTO {
            id: base_event.id.clone(),
            title: base_event.title.clone(),
            start_date,
            start_time,
            end_date,
            end_time,
            min_price: if prices.is_empty() { 0.0 } else { min_price },
            max_price: if prices.is_empty() { 0.0 } else { max_price },
        });
    }

    // Return the ApiResponse with the events data
    ApiResponse {
        data: EventsData {
            events: events_data,
        },
        error: None,
    }
}

fn split_datetime(dt: &str) -> (String, String) {
    match dt.split_once('T') {
        Some((date, time)) => (date.to_string(), time.to_string()),
        None => (dt.to_string(), "".to_string()),
    }
}
