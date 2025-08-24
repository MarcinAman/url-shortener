use actix_web::web::{Data, Json};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use actix_web::middleware::Logger;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

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

async fn get_shortened_url(url: String, server_domain: &str, random_part: String) -> String {
    let checksum = crc32fast::hash(url.as_bytes());
    let encoded = base62::encode(checksum);
    // We store old url and the order integer in redis using HSET command
    // We can use the TTL feature for expiration date
    format!("{}/{}{}", server_domain, encoded, random_part)
}

fn generate_random_code(rng: &mut SmallRng) -> String {
    let random_number: u32 = rng.random();
    base62::encode(random_number)
}

async fn resolve_shortened_url(_shortened_url: String) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shortened_url_happy_path() {
        // Arrange
        let test_url = "https://example.com/very/long/url/that/needs/shortening".to_string();
        let test_domain = "https://short.ly";
        let test_random_part = "abc123".to_string();

        // Act - use tokio::runtime to run the async function
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_shortened_url(test_url.clone(), test_domain, test_random_part.clone()));

        // Assert
        assert!(result.starts_with(test_domain));
        assert!(result.contains(&test_random_part));
        
        let parts: Vec<&str> = result.split('/').collect();
        // The last part should contain both checksum and random part concatenated
        let combined_part = parts[3];
        assert!(!combined_part.is_empty());
        assert!(combined_part.chars().all(|c| c.is_alphanumeric()));
        assert!(combined_part.ends_with(&test_random_part));
        
        // Verify the overall format: domain/checksum + random_part
        assert_eq!(result, format!("{}/{}", test_domain, combined_part));
        assert_eq!(result, "https://short.ly/2dHrrEabc123");
    }
}
