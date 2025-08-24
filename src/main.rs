use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use actix_web::middleware::Logger;
use rand::rngs::SmallRng;
use rand::SeedableRng;

mod url_shortener;
use url_shortener::{get_shortened_url, generate_random_code};
mod redis;
use redis::get_redis_service;

use crate::redis::RedisService;

#[get("/{path}")]
async fn resolve(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match state.redis_service.get(&path.into_inner()).await {
        // We can return permanent redirect here, but this would limit our ability to do analytics
        Ok(Some(long_url)) => HttpResponse::TemporaryRedirect()
            .append_header(("Location", long_url))
            .finish(),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(err) => {
            log::error!("Failed to get long URL from Redis: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
struct UrlShortenOptions {
    url: String
}

#[derive(Serialize)]
struct UrlShortenData {
    short_url: String,
}

#[post("/shorten-url")]
async fn shorten_url(req_body: Json<UrlShortenOptions>, state: Data<AppState>) -> impl Responder {
    let url = req_body.0.url.clone();
    let shortened_data = UrlShortenData {
        short_url: get_shortened_url(url.clone(), &state.domain, generate_random_code(&mut state.rng.clone())).await,
    };
    //TODO: Collision detection and proper error handling for those cases
    let save_result = state.redis_service.set(&shortened_data.short_url, &url, Some(60 * 60 * 24)).await;
    if save_result.is_err() {    
        log::error!("Failed to save shortened URL: {}", save_result.err().unwrap());
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().json(shortened_data)
}

struct AppState {
    domain: String,
    rng: SmallRng,
    redis_service: RedisService,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let state = Data::new(
        AppState {
            domain: "https://short.me".to_string(),
            rng: SmallRng::from_os_rng(),
            redis_service: get_redis_service().await.unwrap(),
        });

    HttpServer::new(move || {
        App::new()
            .service(resolve)
            .service(shorten_url)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(state.clone())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
