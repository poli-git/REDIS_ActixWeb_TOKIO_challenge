
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use storage::connections::cache::connect;
use redis::Commands;

async fn get_redis_keys() -> impl Responder {
    let mut conn = match connect() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Redis connection error: {}", e)),
    };

    let keys: redis::RedisResult<Vec<String>> = conn.keys("*");
    match keys {
        Ok(keys) => HttpResponse::Ok().json(keys),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize dotenv and logging
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting webapp on 0.0.0.0:8080");
    HttpServer::new(|| {
        App::new()
            .route("/redis-keys", web::get().to(get_redis_keys))
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}