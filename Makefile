include .env
export

.PHONY: all check-rust-version build-plugin validate-plugin build-run run stop clean

# Default target
all: check-rust-version build-plugin run

# Check Rust version
check-rust-version:
	@echo "Checking Rust version..."
	@if command -v rustc >/dev/null 2>&1; then \
		current_version=$$(rustc -V | awk '{print $$2}'); \
		if [ "$$current_version" != "$(RUST_VERSION)" ]; then \
			echo "Error: Rust version $(RUST_VERSION) is required, but $$current_version is installed."; \
			echo "Please install the correct version or use rustup to switch versions."; \
			exit 1; \
		else \
			echo "Rust version $(RUST_VERSION) is installed."; \
		fi \
	else \
		echo "Error: Rust is not installed. Please install Rust $(RUST_VERSION)."; \
		exit 1; \
	fi

# Generate rust-toolchain.toml
generate-rust-toolchain:
	@./generate-rust-toolchain.sh

# Generate SSL certificates for local testing
generate-ssl-certs:
	@./generate-ssl-certs.sh

# Tests the WASM plugin
test-plugin: check-rust-version generate-rust-toolchain
	@echo "Testing WASM plugin..."
	@cd rust-http-wasm && cargo test

# Build the WASM plugin
build-plugin: test-plugin
	@echo "Building WASM plugin..."
	@cd rust-http-wasm && ./build.sh

# Validate the WASM plugin binary exports
validate-plugin:
	@./validate-wasm-output.sh

# Run Docker Compose
run: build-plugin validate-plugin
	@echo "Starting services with Docker Compose..."
	docker compose up -d

# Stop Docker Compose
stop:
	@echo "Stopping services with Docker Compose..."
	docker compose down

# Run Docker Compose with Build
build-run: build-plugin validate-plugin
	@echo "Starting services with Docker Compose..."
	docker compose up -d --build

# Clean up
clean:
	@echo "Cleaning up..."
	docker compose down -v --remove-orphans
	docker system prune -af --volumes
	rm -rf plugins-local/src

# Helper target to rebuild and restart
restart: clean all
