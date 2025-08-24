# URL Shortener

A high-performance URL shortening service built with Rust and Actix Web.

## Features

- Fast URL shortening with CRC32 checksums and random codes
- Redis-based storage with TTL support
- RESTful API endpoints
- Comprehensive E2E testing
- Automatic collision resolution with configurable retry attempts
- HTTP 508 status code when collision resolution fails

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- Redis server running on `localhost:6379`

### Running the Service

```bash
# Start Redis (if not already running)
docker-compose up -d redis
# or
brew services start redis

# Run the service
cargo run
```

The service will be available at `http://localhost:8080`

### API Endpoints

- `POST /shorten-url` - Shorten a URL
- `GET /{short_code}` - Redirect to original URL

### Collision Resolution

The service automatically handles URL shortening collisions:
- **Configurable Retry Attempts**: Default 5 attempts (configurable via `max_collision_attempts`)
- **Automatic Regeneration**: Each attempt generates a new random code
- **Error Handling**: Returns HTTP 508 (Loop Detected) if all attempts fail
- **Detailed Error Response**: JSON response with attempt count and error details

## Testing

### E2E Tests

The project includes comprehensive End-to-End tests that verify the complete URL shortening flow:

```bash
# Run all E2E tests
cargo test e2e_tests -- --nocapture

# Run specific test
cargo test test_url_shortening_flow -- --nocapture

# Run tests with single thread (recommended for Redis operations)
cargo test e2e_tests -- --nocapture --test-threads=1
```

### Test Features

- **Automatic Redis Cleanup**: Tests handle Redis cleanup automatically
- **Isolated Testing**: Each test runs in isolation with fresh Redis state
- **Core Logic Testing**: Tests the actual URL shortening and Redis operations
- **Error Handling**: Comprehensive testing of edge cases

### Test Structure

#### E2E Tests
- `test_url_shortening_flow` - Complete URL shortening flow
- `test_url_shortening_with_different_urls` - Multiple URL uniqueness
- `test_nonexistent_short_url` - Error handling for missing URLs

#### RedisService Unit Tests
- `test_redis_service_set_then_get` - Tests Redis set and get operations
- `test_redis_service_get_nonexistent_key` - Tests handling of missing keys
- `test_redis_service_set_nx_prevents_overwrite` - Tests that NX flag prevents key overwriting (returns false)
- `test_redis_service_ttl_functionality` - Tests TTL expiration behavior

## Development

### Project Structure

```
src/
├── main.rs          # Main application and E2E tests
├── url_shortener.rs # URL shortening logic
└── redis.rs         # Redis service implementation
```

### Adding New Tests

To add new E2E tests, use the provided setup and teardown functions:

```rust
#[tokio::test]
async fn test_new_feature() {
    let test_app = setup_test().await;
    
    // Test implementation here
    
    teardown_test(test_app).await;
}
```

## Documentation

For detailed testing information, see [E2E_TESTING.md](E2E_TESTING.md).
