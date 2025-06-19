use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use actix_web::middleware::Logger;

#[get("/{path}")]
async fn hello(path: web::Path<String>, _state: web::Data<AppState>) -> impl Responder {
    let Some(long_url) = resolve_shortened_url(path.into_inner()).await else {
        return HttpResponse::NotFound().finish();
    };

    HttpResponse::PermanentRedirect()
        .append_header(("Location", long_url))
        .finish()
}


#[derive(Deserialize)]
struct UrlShortenOptions {
    url: String,
}

#[derive(Serialize)]
struct UrlShortenData {
    short_url: String,
}

#[post("/shorten-url")]
async fn shorten_url(req_body: Json<UrlShortenOptions>, state: Data<AppState>) -> impl Responder {
    let shortened_data = UrlShortenData {
        short_url: get_shortened_url(req_body.0.url, &state.domain, &state.counter).await,
    };
    HttpResponse::Ok().json(shortened_data)
}

struct AppState {
    domain: String,
    counter: Arc<AtomicU64>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let state = Data::new(
        AppState {
            domain: "https://short.ly".to_string(),
            counter: Arc::new(AtomicU64::new(0)),
        });

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(shorten_url)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(state.clone())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

async fn get_shortened_url(_url: String, server_domain: &str, counter: &Arc<AtomicU64>) -> String {
    let order = counter.fetch_add(1, SeqCst);
    format!("{}/{}", server_domain, base62::encode(order))
}

async fn resolve_shortened_url(_shortened_url: String) -> Option<String> {
    None
}
