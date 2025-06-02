use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "eventList")]
pub struct EventList {
    pub output: Output,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    #[serde(rename = "base_event")]
    pub base_events: Vec<BaseEvent>,
}

#[derive(Debug, Deserialize)]
pub struct BaseEvent {
    #[serde(rename = "@base_event_id")]
    pub base_event_id: Option<u32>,
    #[serde(rename = "@sell_mode")]
    pub sell_mode: Option<String>,
    #[serde(rename = "@organizer_company_id")]
    pub organizer_company_id: Option<u32>,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "event")]
    pub event: Event,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(rename = "@event_start_date")]
    pub event_start_date: String,
    #[serde(rename = "@event_end_date")]
    pub event_end_date: String,
    #[serde(rename = "@event_id")]
    pub event_id: Option<u32>,
    #[serde(rename = "@sell_from")]
    pub sell_from: Option<String>,
    #[serde(rename = "@sell_to")]
    pub sell_to: Option<String>,
    #[serde(rename = "@sold_out")]
    pub sold_out: Option<bool>,
    #[serde(rename = "zone")]
    pub zones: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    #[serde(rename = "@zone_id")]
    pub zone_id: Option<u32>,
    #[serde(rename = "@capacity")]
    pub capacity: Option<u32>,
    #[serde(rename = "@price")]
    pub price: Option<f32>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@numbered")]
    pub numbered: Option<bool>,
}
