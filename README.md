# URL Shortener

A high-performance URL shortening service built with Rust and Actix Web.

## Features

- Fast URL shortening with CRC32 checksums and random codes
- Redis-based storage with TTL support
- RESTful API endpoints
- Comprehensive E2E testing
- Automatic collision resolution with configurable retry attempts
- HTTP 508 status code when collision resolution fails

## How Short URLs Are Generated

The service generates short URLs using a combination of CRC32 checksums and random codes:

1. **CRC32 Checksum**: A 32-bit checksum is computed from the original URL
2. **Random Code Generation**: A random 6-character alphanumeric code is generated
3. **Collision Resolution**: If a collision occurs, the service automatically generates new random codes up to a configurable limit
4. **Final Short Code**: The final short code is a combination that ensures uniqueness and limit the ability for "guessing" the url.

This approach provides:
- **Fast Generation**: CRC32 is computationally efficient
- **Uniqueness**: Random codes minimize collision probability
- **Reliability**: Automatic collision resolution handles edge cases
- **Performance**: O(1) average time complexity for URL shortening

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

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests with single thread (recommended for Redis operations)
cargo test -- --nocapture --test-threads=1
```

## Development

### Project Structure

```
src/
├── main.rs          # Main application and E2E tests
├── url_shortener.rs # URL shortening logic
└── redis.rs         # Redis service implementation
```

## Documentation

For detailed testing information, see [E2E_TESTING.md](E2E_TESTING.md).
