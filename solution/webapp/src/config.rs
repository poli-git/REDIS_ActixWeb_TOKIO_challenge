use dotenv::dotenv;
use serde::Deserialize;

fn actix_client_shutdown_ms() -> u64 {
    5000
}

fn actix_client_timeout_ms() -> u64 {
    5000
}

fn actix_shutdown_timeout_s() -> u64 {
    30
}

fn actix_keepalive_seconds() -> u64 {
    5
}
fn actix_num_workers() -> usize {
    4
}

fn web_app_server() -> String {
    "127.0.0.1:8080".to_string()
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "actix_client_shutdown_ms")]
    pub actix_client_shutdown_ms: u64,

    #[serde(default = "actix_client_timeout_ms")]
    pub actix_client_timeout_ms: u64,

    #[serde(default = "actix_shutdown_timeout_s")]
    pub actix_shutdown_timeout_s: u64,

    #[serde(default = "actix_keepalive_seconds")]
    pub actix_keepalive_seconds: u64,

    #[serde(default = "actix_num_workers")]
    pub actix_num_workers: usize,

    #[serde(default = "web_app_server")]
    pub web_app_server: String,
}

pub fn build() -> Config {
    dotenv().ok();
    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Error reading config values {:#?}", error),
    }
}
