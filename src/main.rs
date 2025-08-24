use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use actix_web::middleware::Logger;
use rand::rngs::SmallRng;
use rand::SeedableRng;

mod url_shortener;
use url_shortener::{get_shortened_url, generate_random_code, resolve_shortened_url};

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
    url: String
}

#[derive(Serialize)]
struct UrlShortenData {
    short_url: String,
}

#[post("/shorten-url")]
async fn shorten_url(req_body: Json<UrlShortenOptions>, state: Data<AppState>) -> impl Responder {
    let shortened_data = UrlShortenData {
        short_url: get_shortened_url(req_body.0.url, &state.domain, generate_random_code(&mut state.rng.clone())).await,
    };
    HttpResponse::Ok().json(shortened_data)
}

struct AppState {
    domain: String,
    rng: SmallRng,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let state = Data::new(
        AppState {
            domain: "https://short.ly".to_string(),
            rng: SmallRng::from_os_rng(),
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
