# Treblle's Traefik (`^v3.1`) Middleware Plugin in Rust and WebAssembly

This project implements a Traefik v3 middleware plugin using Rust and WebAssembly (WASM). The plugin intercepts HTTP requests, processes them, and forwards relevant data to the Treblle API for monitoring and analysis.

## Project Structure

The project consists of several components:

1. **Traefik Middleware Plugin (`rust-http-wasm`)**: A Rust-based WASM module that integrates with Traefik v3.
2. **Producer Service (`producer`)**: Generates random HTTP requests.
3. **Consumer Service (`consumer`)**: Receives and processes the requests.
4. **Treblle API (`treblle-api`)**: A mock API that receives and logs the processed data.

## Prerequisites

- Docker and Docker Compose
- Rust (version specified in `.env` file)
- rustup
- Homebrew (if on macOS)
- make

## Setup and Installation

1. Clone this repository:

   ```sh
   git clone https://github.com/your-repo/treblle-traefik-plugin.git
   cd treblle-traefik-plugin
   ```

1. Install required tools:
   - If you're on macOS and don't have Homebrew, install it from [brew.sh](https://brew.sh/).
   - Install CMake and Ninja (required for building wabt):

     ```sh
     brew install cmake ninja
     ```

   - Optionally install pthread-stubs for threading support:

     ```sh
     brew install pthread-stubs
     ```

1. Install Rust and required components:
   - The project uses a specific Rust version defined in the `.env` file.
   - The `rust-toolchain.toml` file in the root of the repository helps manage the Rust version and components.
   - If you don't have rustup, install it from [rustup.rs](https://rustup.rs/).

1. Install WASM tools:

   ```sh
   ./wasm-tools-install.sh
   ```

1. Build and run the project:

   ```sh
   make all
   ```

## Makefile Targets

The project uses a Makefile to manage building and running. Here are the main targets:

- `make all`: Default target. Checks Rust version, builds the plugin, and runs the services.
- `make check-rust-version`: Verifies the installed Rust version matches the required version.
- `make generate-rust-toolchain`: Generates the `rust-toolchain.toml` file.
- `make build-plugin`: Builds the WASM plugin.
- `make validate-plugin`: Validates the WASM plugin exports.
- `make run`: Runs the Docker Compose services.
- `make build-run`: Builds and runs the Docker Compose services.
- `make clean`: Cleans up Docker resources and build artifacts.
- `make restart`: Cleans up and rebuilds everything.

## Configuration

### Traefik Configuration

The Traefik configuration is defined in `traefik.yml` and `traefik_dynamic.yml`. You can modify these files to adjust Traefik's behavior and the plugin settings.

### Plugin Configuration

The plugin configuration is located in `traefik_dynamic.yml` under the `http.middlewares.treblle-middleware.plugin.treblle` section. You can adjust the following settings:

- `treblleApiUrl`: URL of the Treblle API
- `apiKey`: Your Treblle API key
- `projectId`: Your Treblle project ID
- `routeBlacklist`: List of routes to exclude from processing
- `sensitiveKeysRegex`: Regex pattern for masking sensitive data
- `allowedContentType`: Content type to process (default: "application/json")

## Usage

Once the services are running, the Producer service will start generating random HTTP requests to the Consumer service. The Traefik middleware will intercept these requests, process them, and forward the data to the Treblle API.

You can monitor the Traefik dashboard at `http://localhost:8080/dashboard` and check the logs of each service for more information.

## Development

### Updating Rust Version

To update the Rust version used in the project:

1. Modify the `RUST_VERSION` in the `.env` file.
2. Run `make generate-rust-toolchain` to update the `rust-toolchain.toml` file.
3. Run `make check-rust-version` to verify the installed Rust version.

### Modifying the WASM Plugin

The WASM plugin source code is located in the `rust-http-wasm` directory. After making changes:

1. Run `make build-plugin` to rebuild the plugin.
2. Run `make validate-plugin` to ensure the required exports are present.
3. Use `make restart` to apply the changes to the running services.

## Troubleshooting

If you encounter any issues:

1. Check the logs of each service using `docker-compose logs [service_name]`.
2. Ensure all services are running with `docker-compose ps`.
3. Verify the Traefik configuration in `traefik.yml` and `traefik_dynamic.yml`.
4. Check the Rust code in the middleware and Treblle API for any errors.
5. Run `make check-rust-version` to ensure you're using the correct Rust version.
6. If you've modified the WASM plugin, run `make validate-plugin` to check for required exports.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.
