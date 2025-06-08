use serde::{Deserialize, Serialize};
use std::fmt;
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
    pub sell_mode: Option<SellModeEnum>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SellModeEnum {
    Online,
    Offline,
}

impl fmt::Display for SellModeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SellModeEnum::Online => write!(f, "online"),
            SellModeEnum::Offline => write!(f, "offline"),
        }
    }
}
