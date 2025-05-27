use log::info;

/// Returns either the value of the `ACTIX_NUM_WORKERS` env var, or a fallback value of 10.
fn default_database_pool_max_size() -> u32 {
    let fallback = 10;
    std::env::var("DATABASE_POOL_MAX_SIZE").map_or_else(
        |_| {
            info!(
                "No value found for `DATABASE_POOL_MAX_SIZE`.
                 Using default connection pool size of {}",
                fallback
            );
            fallback
        },
        |val| val.parse::<u32>().unwrap_or(fallback),
    )
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_database_pool_max_size")]
    pub database_pool_max_size: u32,
    pub database_url: String,
    pub disable_api_key_hashing: bool,
    pub postgres_url: String,
}

fn build() -> Config {
    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("error reading config {:#?}", error),
    }
}

lazy_static! {
#[derive(Debug)]
    pub static ref CONFIG: Config = build();
}
