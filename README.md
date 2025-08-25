# URL Shortener

A high-performance URL shortening service built with Rust and Actix Web.

## Features

- Fast URL shortening with CRC32 checksums and optionally random codes. CRC32 returns u32 which is plenty for most cases (up to 6 digits of base62 which means 62^6 slots and we also have the random part which extends this range).
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

     checks.........................: 100.00%  ✓ 41870422      ✗ 0       
     data_received..................: 6.6 GB   22 MB/s
     data_sent......................: 3.8 GB   13 MB/s
     http_req_blocked...............: avg=9.13µs   min=167ns     med=625ns   max=192.31ms p(90)=1.16µs  p(95)=1.54µs  
     http_req_connecting............: avg=3.31µs   min=0s        med=0s      max=104.43ms p(90)=0s      p(95)=0s      
   ✓ http_req_duration..............: avg=17.42ms  min=61.54µs   med=16.31ms max=272.37ms p(90)=24.71ms p(95)=30.61ms 
       { expected_response:true }...: avg=17.42ms  min=61.54µs   med=16.31ms max=272.37ms p(90)=24.71ms p(95)=30.61ms 
   ✓ http_req_failed................: 0.00%    ✓ 0             ✗ 41870422
     http_req_receiving.............: avg=243.69µs min=-567097ns med=7.45µs  max=212.23ms p(90)=14.25µs p(95)=42µs    
     http_req_sending...............: avg=24.4µs   min=-475098ns med=2.91µs  max=182.52ms p(90)=5.33µs  p(95)=14.79µs 
     http_req_tls_handshaking.......: avg=0s       min=0s        med=0s      max=0s       p(90)=0s      p(95)=0s      
     http_req_waiting...............: avg=17.15ms  min=51.41µs   med=16.26ms max=208.83ms p(90)=24.54ms p(95)=29.76ms 
     http_reqs......................: 41870422 139553.376017/s
     iteration_duration.............: avg=78.78ms  min=1ms       med=74.21ms max=444.1ms  p(90)=104.6ms p(95)=120.11ms
     iterations.....................: 3806402  12686.670547/s
     vus............................: 1000     min=1000        max=1000  
     vus_max........................: 1000     min=1000        max=1000  


running (5m00.0s), 0000/1000 VUs, 3806402 complete and 0 interrupted iterations
default ✓ [======================================] 1000 VUs  5m0s
```