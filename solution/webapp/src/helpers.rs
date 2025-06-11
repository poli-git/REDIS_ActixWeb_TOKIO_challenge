use serde::Serialize;
use storage::connections::cache::ProviderABaseEvent;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct EventsData {
    pub events: Vec<EventDTO>,
}

#[derive(Serialize)]
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

pub fn map_provider_events_to_response_dto(
    base_events: &[ProviderABaseEvent],
) -> ApiResponse<EventsData> {
    let mut events_data = Vec::new();

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
