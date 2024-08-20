# Treblle's Traefik (`^v3.1`) Middleware Plugin in Rust and WebAssembly

## Overview

This project provides a middleware plugin for Traefik that integrates Treblle's API monitoring and logging services. The plugin collects data from Traefik's request/response lifecycle, masks sensitive information, and sends the sanitized data to Treblle's API for monitoring. This plugin is designed to be lightweight, efficient, and easy to install through the Traefik catalog.

## Features

- **Data Ingestion:** Captures request and response data from Traefik and sends it to Treblle via a POST request in JSON format.
- **Sensitive Data Masking:** Automatically masks sensitive data such as passwords, credit card numbers, and other user-defined fields before sending data to Treblle.
  - **Customizable Masking:** Users can define additional custom keywords for masking sensitive data.
- **Route Blacklisting:** Allows users to define specific routes or regex patterns to exclude from data collection and reporting.
- **Silent Error Handling:** Ensures that any errors in the plugin do not interfere with the host application's functionality.
- **Fire-and-Forget:** The plugin sends data without waiting for a response, ensuring minimal impact on performance.
- **WASM-WASI1P Compatible**: Built using WebAssembly (WASM) for high performance and compatibility with support for outgoing HTTP requests.

## Project Structure

The project consists of several components:

1. **Traefik Middleware Plugin (`rust-http-wasm`)**: A Rust-based WASM module that integrates with Traefik v3.1 or newer.
   - This version is important because Traefik v3.1 introduced enhanced support for WASM plugins required by this project.
1. **Producer Service (`producer`)**: Generates various HTTP requests, including JSON, plain text, and XML.
1. **Consumer Service (`consumer`)**: Receives and processes the requests, including handling different routes.
1. **Treblle API (`treblle-api`)**: A mock API that receives and logs the processed data.

## Prerequisites

- Docker and Docker Compose
- Rust (version specified in `.env` file)
- `rustup`
- Homebrew (if on macOS)
- `make`

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
- `routeBlacklist`: List of routes to exclude from processing (e.g., ["/blacklisted-example"])
- `sensitiveKeysRegex`: Regex pattern for masking sensitive data

#### Example configuration

```yml
http:
  middlewares:
    my-treblle-middleware:
      plugin:
        treblle:
            apiKey: "your_api_key_here"
            projectId: "your_project_id_here"
            routeBlacklist:
               ["/ping", "/healthcheck", "/blacklisted-example"]
            sensitiveKeysRegex: "(?i)(password|pwd|secret|password_confirmation|cc|card_number|ccv|ssn|credit_score)"
```

## Usage

Once the services are running:

1. The Producer service will start generating random HTTP requests to the Consumer service, including:
   - JSON requests (with and without sensitive payloads) to `/consume`
   - Plain text requests to `/consume`
   - XML requests to `/consume`
   - JSON requests to `/blacklisted-example` (which should be ignored by the middleware)

1. The Traefik middleware will intercept these requests, process them, and forward the data to the Treblle API if:
   - The request is not to a blacklisted route
   - The content type is JSON

1. The Consumer service will log all received requests, regardless of their processing by the middleware.

1. The Treblle API will receive and log the processed data from valid requests, as well as prepare Prometheus metrics for ingestion.

## Development

### Updating Rust Version

To update the Rust version used in the project:

1. Modify the `RUST_VERSION` in the `.env` file.
1. Run `make generate-rust-toolchain` to update the `rust-toolchain.toml` file.
1. Run `make check-rust-version` to verify the installed Rust version.

### Modifying the WASM Plugin

The WASM plugin source code is located in the `rust-http-wasm` directory. After making changes:

1. Run `make build-plugin` to rebuild the plugin.
1. Run `make validate-plugin` to ensure the required exports are present.
1. Use `make restart` to apply the changes to the running services.

## Request Body Handling

The middleware plugin implements a specific pattern for handling request bodies, which is important to understand:

1. **Reading the Request Body**:
   The middleware reads the request body using the `host_read_request_body` function. This operation "consumes" the body, meaning it's no longer available for subsequent middleware or the final handler (Consumer service).

1. **Processing the Body**:
   After reading, the body is processed (e.g., sent to the Treblle API for analysis).

1. **Writing the Body Back**:
   To ensure the original request body is available for the rest of the request processing pipeline, the middleware writes the body back using the `host_write_request_body` function.

### Why is this necessary?

- **Body Consumption**: In many HTTP proxy systems, reading the body often involves buffering the entire content, which can be resource-intensive for large payloads. Once read, the original stream is typically closed to free up resources.

- **WASM Module Isolation**: The WASM module runs in an isolated environment. Reading the body copies the data into the module's memory, triggering the consumption in the host environment (Traefik).

- **Preserving Original Behavior**: By writing the body back, we ensure that subsequent middleware and the final handler can access the body content, maintaining the expected behavior of the HTTP request.

### Implementation Details

The relevant code for this process is in the `handle_request` function:

```rust
let body = host_read_request_body().unwrap_or_else(|_| "{}".to_string());

// ... process the body ...

if let Err(e) = host_write_request_body(body.as_bytes()) {
    host_log(
        LOG_LEVEL_ERROR,
        &format!("Error setting request body back: {}", e),
    );
}
```

This pattern allows the middleware to inspect and potentially modify the body while ensuring that the original (or modified) body is still available for the rest of the request processing pipeline.

## Testing

The project includes several test scenarios:

1. **JSON Requests**: Sent to `/consume`, these should be processed by the middleware and forwarded to the Treblle API.
2. **Non-JSON Requests**: Sent to `/consume`, these should be ignored by the middleware but still reach the Consumer.
3. **Blacklisted Route**: Requests to `/blacklisted-example` should be ignored by the middleware but reach the Consumer.
4. **Sensitive Data**: JSON requests containing sensitive information (like passwords or credit card numbers) should be masked before being sent to the Treblle API.

To verify these scenarios, check the logs of the Consumer service and the Treblle API after running the system for a while.

## Troubleshooting

If you encounter any issues:

1. Check the logs of each service using `docker-compose logs [service_name]`.
1. Ensure all services are running with `docker-compose ps`.
1. Verify the Traefik configuration in `traefik.yml` and `traefik_dynamic.yml`.
1. Check the Rust code in the middleware and Treblle API for any errors.
1. Run `make check-rust-version` to ensure you're using the correct Rust version.
1. If you've modified the WASM plugin, run `make validate-plugin` to check for required exports.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.
