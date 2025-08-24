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

## Performance testing with k6

Minimal Docker-based setup is provided to run a GET-heavy workload (default 10:1 GET:POST) against the service.

### One-time build and start

```bash
make docker-up
```

This builds the app container and starts `redis` and `app`. The app listens on `0.0.0.0:8080`.

### Run the k6 test

```bash
# Default parameters
make perf
```

Performance numbers for a single pod setup run on MBP 13" with M3 Max:
```
docker compose run --rm -e VUS=1000 -e DURATION=5m k6
[+] Creating 2/2
 ✔ Container url-shortener-redis-1  Running                                                               0.0s 
 ✔ Container url-shortener-app-1    Running                                                               0.0s 

          /\      |‾‾| /‾‾/   /‾‾/   
     /\  /  \     |  |/  /   /  /    
    /  \/    \    |     (   /   ‾‾\  
   /          \   |  |\  \ |  (‾)  | 
  / __________ \  |__| \__\ \_____/ .io

     execution: local
        script: /scripts/perf.js
        output: -

     scenarios: (100.00%) 1 scenario, 1000 max VUs, 5m30s max duration (incl. graceful stop):
              * default: 1000 looping VUs for 5m0s (gracefulStop: 30s)


     ✓ post status is 200
      ↳  99% — ✓ 3539513 / ✓ 1461
     ✓ get status is 307

     checks.........................: 99.99%   ✓ 38934643     ✗ 1461    
     data_received..................: 5.6 GB   19 MB/s
     data_sent......................: 3.7 GB   12 MB/s
     http_req_blocked...............: avg=6.58µs   min=0s        med=667ns   max=340.96ms p(90)=1.29µs   p(95)=1.7µs   
     http_req_connecting............: avg=1.28µs   min=0s        med=0s      max=275.5ms  p(90)=0s       p(95)=0s      
   ✓ http_req_duration..............: avg=18.18ms  min=65.16µs   med=17.05ms max=308.93ms p(90)=26.28ms  p(95)=31.96ms 
       { expected_response:true }...: avg=18.18ms  min=65.16µs   med=17.05ms max=308.93ms p(90)=26.27ms  p(95)=31.96ms 
   ✓ http_req_failed................: 0.00%    ✓ 1461         ✗ 38934643
     http_req_receiving.............: avg=255.32µs min=-291056ns med=7.58µs  max=270.69ms p(90)=15.37µs  p(95)=46.66µs 
     http_req_sending...............: avg=26.7µs   min=-259681ns med=3.12µs  max=278.38ms p(90)=5.91µs   p(95)=16.58µs 
     http_req_tls_handshaking.......: avg=0s       min=0s        med=0s      max=0s       p(90)=0s       p(95)=0s      
     http_req_waiting...............: avg=17.89ms  min=46.91µs   med=16.99ms max=234.51ms p(90)=26.09ms  p(95)=31.08ms 
     http_reqs......................: 38936104 129770.88357/s
     iteration_duration.............: avg=84.68ms  min=2.61ms    med=80.12ms max=517.35ms p(90)=113.17ms p(95)=128.79ms
     iterations.....................: 3540974  11801.779774/s
     vus............................: 1000     min=1000       max=1000  
     vus_max........................: 1000     min=1000       max=1000  


running (5m00.0s), 0000/1000 VUs, 3540974 complete and 0 interrupted iterations
default ✓ [======================================] 1000 VUs  5m0s
```
