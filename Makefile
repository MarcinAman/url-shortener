.PHONY: bench run

bench:
	wrk -t12 -c50 -d30s -s benchmark_post_request.lua http://localhost:8080/shorten-url

run:
	RUST_LOG=actix_web=error cargo run --release

run-debug:
	RUST_BACKTRACE=1 RUST_LOG=actix_web=debug cargo run

docker-up:
	docker compose up -d --build redis app

perf:
	docker compose run --rm -e SAVES=$(SAVES) -e RATIO=$(RATIO) -e VUS=$(VUS) k6

help:
	@echo "Available commands:"
	@echo "  bench       - Run benchmark against localhost:8080"
	@echo "  run         - Run the application"
	@echo "  run-debug   - Run the application in debug mode"
	@echo "  docker-up   - Start redis and app via Docker"
	@echo "  perf        - Run k6 perf test (env: SAVES, RATIO, VUS)"
