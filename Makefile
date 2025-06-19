.PHONY: bench run

bench:
	wrk -t12 -c50 -d30s -s benchmark_post_request.lua http://localhost:8080/shorten-url

run:
	RUST_LOG=actix_web=error cargo run --release

run-debug:
	RUST_BACKTRACE=1 RUST_LOG=actix_web=debug cargo run

help:
	@echo "Available commands:"
	@echo "  bench       - Run benchmark against localhost:8080"
	@echo "  run         - Run the application"
	@echo "  run-debug   - Run the application in debug mode"
