include .env
export

.PHONY: all check-rust-version build-plugin validate-plugin build-run run clean

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

# Build the WASM plugin
build-plugin: check-rust-version generate-rust-toolchain
	@echo "Building WASM plugin..."
	@cd rust-http-wasm && ./build.sh

# Validate the WASM plugin exports
validate-plugin:
	@./validate-wasm-output.sh

# Run Docker Compose
run: build-plugin validate-plugin
	@echo "Starting services with Docker Compose..."
	docker-compose up -d

# Run Docker Compose with Build
build-run: build-plugin validate-plugin
	@echo "Starting services with Docker Compose..."
	docker-compose up -d --build

# Clean up
clean:
	@echo "Cleaning up..."
	docker-compose down -v --remove-orphans
	docker system prune -af --volumes
	rm -rf plugins-local/src

# Helper target to rebuild and restart
restart: clean all