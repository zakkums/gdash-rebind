.PHONY: help build run test bench clean

help:
	@echo "Available targets:"
	@echo "  make build  - Build release binary (target/release/kmrebind)"
	@echo "  make run    - Build and run kmrebind"
	@echo "  make test   - Run unit and integration tests"
	@echo "  make bench  - Run latency/throughput benchmarks"
	@echo "  make clean  - Remove target/ and build artifacts"

build:
	cargo build --release

run: build
	./target/release/kmrebind $(ARGS)

test:
	cargo test

bench:
	cargo bench

clean:
	rm -rf target/
