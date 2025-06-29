use dotenv::dotenv;
use serde::Deserialize;

fn async_worker_interval_sec() -> u32 {
    300
}

fn redis_expiration_key_time_limit_sec() -> u32 {
    3600
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "async_worker_interval_sec")]
    pub async_worker_interval_sec: u32,

    #[serde(default = "redis_expiration_key_time_limit_sec")]
    pub redis_expiration_key_time_limit_sec: u32,
}

pub fn build() -> Config {
    dotenv().ok();
    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Error reading config values {:#?}", error),
    }
}
