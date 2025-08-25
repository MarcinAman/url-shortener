use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

mod url_shortener;
use url_shortener::{generate_random_code, get_url_slug};
mod redis;
use redis::get_redis_service;

use crate::redis::RedisService;

#[get("/{path}")]
async fn resolve(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match state.redis_service.get(&path.into_inner()).await {
        // We can return permanent redirect here, but this would limit our ability to do analytics
        Ok(Some(long_url)) => HttpResponse::TemporaryRedirect()
            .append_header(("Location", format!("{}/{}", state.domain, long_url)))
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
    url: String,
}

#[derive(Serialize)]
struct UrlShortenData {
    short_url: String,
}

#[derive(Serialize)]
struct CollisionErrorResponse {
    error: String,
    message: String,
    attempts: u32,
    url: String,
}

#[post("/shorten-url")]
async fn shorten_url(req_body: Json<UrlShortenOptions>, state: Data<AppState>) -> impl Responder {
    let url = req_body.0.url.clone();

    // Try to generate a unique short URL with collision resolution
    let mut attempts = 0;
    let mut short_url = String::new();
    let mut collision_detected = false;
    let mut rng = SmallRng::from_os_rng();

    while attempts < state.max_collision_attempts {
        attempts += 1;

        // Generate a new short URL
        short_url = if attempts == 1 {
            get_url_slug(url.clone(), None).await
        } else {
            get_url_slug(url.clone(), Some(generate_random_code(&mut rng))).await
        };

        // Try to save the short URL
        let save_result = state
            .redis_service
            .set(short_url.as_str(), &url, Some(60 * 60 * 24))
            .await;

        match save_result {
            Ok(true) => {
                // Successfully saved, no collision
                collision_detected = false;
                break;
            }
            Ok(false) => {
                // Collision detected, key already exists
                collision_detected = true;
                log::warn!(
                    "Collision detected on attempt {} for URL: {}",
                    attempts,
                    url
                );

                if attempts >= state.max_collision_attempts {
                    log::error!(
                        "Failed to generate unique short URL after {} attempts for URL: {}",
                        state.max_collision_attempts,
                        url
                    );
                    return HttpResponse::build(StatusCode::LOOP_DETECTED)
                        .json(CollisionErrorResponse {
                            error: "Failed to generate unique short URL".to_string(),
                            message: format!("Unable to generate a unique shortened URL after {} attempts. Please try again later.", state.max_collision_attempts),
                            attempts: state.max_collision_attempts,
                            url: url.clone(),
                        });
                }
                // Continue to next attempt
            }
            Err(e) => {
                // Redis error occurred
                log::error!("Failed to save shortened URL: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    // If we get here, we either succeeded or hit max attempts
    if collision_detected && attempts >= state.max_collision_attempts {
        return HttpResponse::build(StatusCode::LOOP_DETECTED)
            .json(CollisionErrorResponse {
                error: "Failed to generate unique short URL".to_string(),
                message: format!("Unable to generate a unique shortened URL after {} attempts. Please try again later.", state.max_collision_attempts),
                attempts: state.max_collision_attempts,
                url: url.clone(),
            });
    }

    let shortened_data = UrlShortenData {
        short_url: format!("{}/{}", state.domain, short_url),
    };
    HttpResponse::Ok().json(shortened_data)
}

struct AppState {
    domain: String,
    redis_service: RedisService,
    max_collision_attempts: u32,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    log::info!("Starting URL Shortener service");
    let state = Data::new(AppState {
        domain: "https://short.me".to_string(),
        redis_service: get_redis_service().await.unwrap(),
        max_collision_attempts: 5, // Allow 5 attempts to generate a unique short URL
    });

    log::info!("HTTP server binding on 0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .service(resolve)
            .service(shorten_url)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(state.clone())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod e2e_tests {
    use super::*;

    struct TestApp {
        redis_service: RedisService,
    }

    impl TestApp {
        async fn new() -> Self {
            // Create a fresh Redis service for testing
            let redis_service = RedisService::new("redis://localhost:6379")
                .await
                .expect("Failed to connect to Redis");

            TestApp { redis_service }
        }

        async fn cleanup_redis(&self) {
            // Clean up all test data from Redis
            let _ = self.redis_service.cleanup().await;
        }
    }

    impl Clone for TestApp {
        fn clone(&self) -> Self {
            TestApp {
                redis_service: self.redis_service.clone(),
            }
        }
    }

    // Test setup and teardown functions
    async fn setup_test() -> TestApp {
        let test_app = TestApp::new().await;
        // Clean up Redis before each test
        test_app.cleanup_redis().await;
        test_app
    }

    async fn teardown_test(test_app: TestApp) {
        // Clean up Redis after each test
        test_app.cleanup_redis().await;
    }

    #[tokio::test]
    async fn test_url_shortening_flow() {
        let test_app = setup_test().await;

        // Use a real external URL for testing
        let target_url = "https://httpbin.org/get";

        // Step 1: Test URL shortening logic directly
        let shortened_url = get_url_slug(
            target_url.to_string(),
            Some(generate_random_code(&mut SmallRng::from_os_rng())),
        )
        .await;

        // Verify the shortened URL format
        assert!(!shortened_url.contains(&target_url));

        let save_result = test_app
            .redis_service
            .set(&shortened_url, target_url, Some(60 * 60 * 24))
            .await;
        assert!(save_result.is_ok());
        assert!(
            save_result.unwrap(),
            "Key should have been set successfully"
        );

        // Step 3: Test Redis retrieval
        let retrieved_url = test_app.redis_service.get(shortened_url.as_str()).await;
        assert!(retrieved_url.is_ok());
        assert_eq!(retrieved_url.unwrap(), Some(target_url.to_string()));

        teardown_test(test_app).await;
    }

    #[tokio::test]
    async fn test_url_shortening_with_different_urls() {
        let test_app = setup_test().await;

        // Test multiple URLs
        let test_urls = vec![
            "https://httpbin.org/get",
            "https://httpbin.org/status/200",
            "https://httpbin.org/headers",
        ];

        for test_url in test_urls {
            // Test URL shortening logic
            let shortened_url = get_url_slug(
                test_url.to_string(),
                Some(generate_random_code(&mut SmallRng::from_os_rng())),
            )
            .await;

            // Extract short code
            // Test Redis storage and retrieval
            let save_result = test_app
                .redis_service
                .set(&shortened_url, test_url, Some(60 * 60 * 24))
                .await;
            assert!(save_result.is_ok());
            assert!(
                save_result.unwrap(),
                "Key should have been set successfully"
            );

            let retrieved_url = test_app.redis_service.get(shortened_url.as_str()).await;
            assert!(retrieved_url.is_ok());
            assert_eq!(retrieved_url.unwrap(), Some(test_url.to_string()));
        }

        teardown_test(test_app).await;
    }

    #[tokio::test]
    async fn test_nonexistent_short_url() {
        let test_app = setup_test().await;

        // Test retrieval of non-existent key
        let retrieved_url = test_app.redis_service.get("nonexistent").await;
        assert!(retrieved_url.is_ok());
        assert_eq!(retrieved_url.unwrap(), None);

        teardown_test(test_app).await;
    }
}
