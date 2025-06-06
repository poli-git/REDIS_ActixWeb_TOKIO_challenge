use reqwest::Client;
use storage::connections::db::establish_connection;

use log::{error, info};
use quick_xml::de::from_str;
use serde::Deserialize;
use storage::base_plan::add_base_plan;
use storage::models::base_plans::NewBasePlan;
use storage::models::plans::NewPlan;
use storage::plan::add_plan;

use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename = "planList")]
pub struct PlanList {
    pub output: Output,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    #[serde(rename = "base_plan")]
    pub base_plan: Vec<BasePlan>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BasePlan {
    #[serde(rename = "@base_plan_id")]
    pub base_plan_id: Option<String>,
    #[serde(rename = "@sell_mode")]
    pub sell_mode: Option<String>,
    #[serde(rename = "@organizer_company_id")]
    pub organizer_company_id: Option<String>,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "plan")]
    pub plans: Vec<Plan>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Plan {
    #[serde(rename = "@plan_start_date")]
    pub plan_start_date: String,
    #[serde(rename = "@plan_end_date")]
    pub plan_end_date: String,
    #[serde(rename = "@plan_id")]
    pub plan_id: Option<String>,
    #[serde(rename = "@sell_from")]
    pub sell_from: Option<String>,
    #[serde(rename = "@sell_to")]
    pub sell_to: Option<String>,
    #[serde(rename = "@sold_out")]
    pub sold_out: Option<bool>,
    #[serde(rename = "zone")]
    pub zones: Vec<Zone>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Zone {
    #[serde(rename = "@zone_id")]
    pub zone_id: Option<String>,
    #[serde(rename = "@capacity")]
    pub capacity: Option<String>,
    #[serde(rename = "@price")]
    pub price: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@numbered")]
    pub numbered: Option<bool>,
}

pub async fn process_provider_events(provider_id: Uuid, provider_name: String, url: String) {
    info!(
        "Fetching events for provider: {} - {}",
        provider_id, provider_name
    );

    // Send GET request to the provider's URL asynchronously
    let client = Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to fetch events from {}: {}", url, e);
            return;
        }
    };

    // Status
    let status = response.status();
    if !status.is_success() {
        error!("HTTP error {} from {}", status, url);
        return;
    }

    // Fetch the XML body as text
    let xml_body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            error!("Failed to read response body from {}: {}", url, e);
            return;
        }
    };

    // Parse XML into PlanList
    let plan_list: PlanList = match from_str(&xml_body) {
        Ok(pl) => pl,
        Err(e) => {
            error!("Failed to parse XML from {}: {}", url, e);
            return;
        }
    };
    // Clone base_plan so it can be used multiple times
    let base_plans = plan_list.output.base_plan.clone();

    // Map PlanList into Vec<NewEvent>
    let events: Vec<NewBasePlan> = base_plans
        .iter()
        .flat_map(|bp| {
            let base_plan_id = bp.base_plan_id.clone().unwrap_or_default();
            bp.plans.iter().map(move |_plan| NewBasePlan {
                base_plans_id: uuid::Uuid::new_v4(),
                providers_id: provider_id,
                event_base_id: base_plan_id.clone(),
                title: bp.title.clone(),
                sell_mode: bp.sell_mode.clone().unwrap_or_default(),
            })
        })
        .collect();

    info!("Fetched {} base_plans from {}", events.len(), url);

    // Get DB connection
    let connection = establish_connection();
    let mut pg_pool = match connection.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get DB connection: {}", e);
            return;
        }
    };

    // Add each BasePlan to the database
    for bp in plan_list.output.base_plan {
        // Insert the base plan
        let new_base_plan = NewBasePlan {
            base_plans_id: uuid::Uuid::new_v4(),
            providers_id: provider_id,
            event_base_id: bp.base_plan_id.clone().unwrap_or_default(),
            title: bp.title.clone(),
            sell_mode: bp.sell_mode.clone().unwrap_or_default(),
        };

        let inserted_base_plan = match add_base_plan(&mut pg_pool, new_base_plan) {
            Ok(inserted) => {
                info!(
                    "Added base_plan: {} : {}",
                    inserted.title, inserted.base_plans_id
                );
                inserted
            }
            Err(e) => {
                error!("Failed to add base_plan: {}", e);
                continue;
            }
        };

        // Insert each plan for this base plan
        for plan in bp.plans {
            let new_plan = NewPlan {
                plans_id: uuid::Uuid::new_v4(),
                base_plans_id: inserted_base_plan.base_plans_id,
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
                    .and_then(|s| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
                    })
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
                sell_to: plan
                    .sell_to
                    .as_ref()
                    .and_then(|s| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
                    })
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
                sold_out: plan.sold_out.unwrap_or(false),
            };

            match add_plan(&mut pg_pool, new_plan) {
                Ok(inserted_plan) => info!(
                    "Added plan: {} : {}",
                    inserted_plan.event_plan_id, inserted_plan.plans_id
                ),
                Err(e) => error!("Failed to add plan: {}", e),
            }
        }
    }
}
