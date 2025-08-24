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
marcinaman@Marcins-MacBook-Pro url-shortener % VUS=1000 DURATION=5m make perf
docker compose run --rm -e VUS=1000 -e DURATION=5m k6
[+] Creating 2/2
 ✔ Container url-shortener-redis-1  Running                              0.0s 
 ✔ Container url-shortener-app-1    R...                                 0.0s 

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
     ✓ get status is 307

     checks.........................: 100.00%  ✓ 40283353     ✗ 0       
     data_received..................: 5.8 GB   19 MB/s
     data_sent......................: 3.8 GB   13 MB/s
     http_req_blocked...............: avg=7.12µs   min=0s        med=625ns   max=132.75ms p(90)=1.25µs   p(95)=1.62µs  
     http_req_connecting............: avg=1.98µs   min=0s        med=0s      max=90.41ms  p(90)=0s       p(95)=0s      
   ✓ http_req_duration..............: avg=17.98ms  min=57µs      med=16.86ms max=205.39ms p(90)=25.68ms  p(95)=31.78ms 
       { expected_response:true }...: avg=17.98ms  min=57µs      med=16.86ms max=205.39ms p(90)=25.68ms  p(95)=31.78ms 
   ✓ http_req_failed................: 0.00%    ✓ 0            ✗ 40283353
     http_req_receiving.............: avg=257.18µs min=-409472ns med=7.5µs   max=154.07ms p(90)=14.16µs  p(95)=42.79µs 
     http_req_sending...............: avg=25.89µs  min=-124931ns med=3.08µs  max=154.03ms p(90)=5.62µs   p(95)=15.66µs 
     http_req_tls_handshaking.......: avg=0s       min=0s        med=0s      max=0s       p(90)=0s       p(95)=0s      
     http_req_waiting...............: avg=17.69ms  min=49.25µs   med=16.81ms max=121.08ms p(90)=25.51ms  p(95)=30.91ms 
     http_reqs......................: 40283353 134252.77366/s
     iteration_duration.............: avg=81.89ms  min=1.43ms    med=77.36ms max=378.89ms p(90)=109.17ms p(95)=124.49ms
     iterations.....................: 3662123  12204.797605/s
     vus............................: 1000     min=1000       max=1000  
     vus_max........................: 1000     min=1000       max=1000  


running (5m00.1s), 0000/1000 VUs, 3662123 complete and 0 interrupted iterations
default ✓ [======================================] 1000 VUs  5m0s
```