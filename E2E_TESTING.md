# E2E Testing for URL Shortener

This document describes the End-to-End (E2E) testing setup for the URL shortener service.

## Overview

The E2E tests verify the complete URL shortening flow by testing the core components:
1. **URL Shortening Logic**: Direct testing of the `get_shortened_url` function
2. **Redis Operations**: Testing storage, retrieval, and cleanup
3. **Error Handling**: Testing edge cases and failure scenarios
4. **Data Integrity**: Ensuring shortened URLs are unique and correctly formatted

**Note**: RedisService-specific tests (set/get operations, key overwriting, TTL functionality) are now unit tests within the `redis.rs` module.

## Test Structure

### TestApp Helper
- **Redis Service**: Fresh Redis connection for each test
- **Cleanup Methods**: Automated Redis cleanup before and after tests
- **Isolated Testing**: No shared state between tests

### Test Setup and Teardown
- **`setup_test()`**: Creates fresh test environment and cleans Redis before each test
- **`teardown_test()`**: Cleans up Redis after each test completes
- **Automatic Cleanup**: Each test automatically gets a clean Redis state

### Test Cases

#### 1. `test_url_shortening_flow`
Tests the complete happy path:
- Tests Redis connection and cleanup
- Generates shortened URL using core logic
- Verifies URL format and uniqueness
- Tests Redis storage and retrieval
- Confirms data integrity

#### 2. `test_url_shortening_with_different_urls`
Tests multiple URLs to ensure uniqueness:
- Shortens multiple different URLs
- Verifies each shortened URL is unique
- Tests Redis operations for each URL
- Ensures no data conflicts

#### 3. `test_nonexistent_short_url`
Tests error handling:
- Attempts to retrieve non-existent short URL
- Verifies proper None response from Redis
- Tests graceful handling of missing data



## Prerequisites

### Dependencies
- Redis server running on `localhost:6379`
- Rust toolchain with Cargo

### Test Dependencies
The following dev-dependencies are added to `Cargo.toml`:
- `reqwest`: HTTP client for potential future HTTP testing
- `tokio-test`: Async testing utilities

## Running Tests

### Run All E2E Tests
```bash
# Ensure Redis is running
redis-cli ping

# Run E2E tests
cargo test e2e_tests -- --nocapture
```

### Run Specific Test
```bash
cargo test test_url_shortening_flow -- --nocapture
```

### Run Tests with Output
```bash
cargo test e2e_tests -- --nocapture --test-threads=1
```

## Redis Cleanup

Redis cleanup is handled entirely from Rust code:

### Automatic Cleanup
- **Before each test**: `setup_test()` calls `FLUSHDB` to ensure clean state
- **After each test**: `teardown_test()` calls `FLUSHDB` to remove test data
- **No manual intervention**: Tests are completely self-contained

### Cleanup Implementation
```rust
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
```

## Test Isolation

- Each test creates its own `TestApp` instance
- Fresh Redis connection per test
- No shared state between tests
- Automatic cleanup prevents test interference

## Test Approach

The current tests focus on testing the core business logic rather than full HTTP integration:

### Core Logic Testing
- Direct function calls to `get_shortened_url`
- Redis operations testing
- Data format validation
- Error handling verification

### Benefits of This Approach
- Faster test execution
- More reliable (no network dependencies)
- Easier debugging
- Better isolation

## Troubleshooting

### Redis Connection Issues
```bash
# Check if Redis is running
redis-cli ping

# Start Redis with Docker
docker-compose up -d redis

# Start Redis with Homebrew (macOS)
brew services start redis
```

### Test Failures
- Check Redis connectivity
- Verify Redis has enough memory for test operations
- Review test output for specific error messages

## Adding New Tests

To add new E2E tests:

1. Create a new test function in the `e2e_tests` module
2. Use `setup_test()` and `teardown_test()` for proper Redis management
3. Follow the pattern of setup → test → teardown
4. Include proper assertions for expected behavior

Example:
```rust
#[tokio::test]
async fn test_new_feature() {
    let test_app = setup_test().await;
    
    // Test implementation here
    
    teardown_test(test_app).await;
}
```

## Future Enhancements

For full HTTP integration testing, consider:
- Adding HTTP server testing with Actix web test utilities
- Implementing mock external services
- Testing complete request/response cycles
- Adding performance and load testing
