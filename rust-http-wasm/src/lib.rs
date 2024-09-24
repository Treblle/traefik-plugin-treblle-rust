//! Treblle middleware for Traefik
//!
//! This module implements a WASM-based middleware for Traefik that integrates
//! with Treblle's API monitoring and logging services.

#![cfg_attr(test, allow(unused_imports, dead_code))]

#[cfg(feature = "wasm")]
mod bindings;

#[cfg(feature = "wasm")]
mod host_functions;

mod config;
mod constants;
mod logger;
mod error;
mod http_client;
mod payload;
mod route_blacklist;
mod schema;
mod utils;

use once_cell::sync::Lazy;

#[cfg(feature = "wasm")]
use bindings::exports::traefik::http_handler::handler::Guest;

#[cfg(feature = "wasm")]
use host_functions::*;

use config::Config;
use logger::{log, LogLevel};
use error::{Result, TreblleError};
use http_client::HttpClient;
use payload::Payload;
use route_blacklist::RouteBlacklist;
use schema::ErrorInfo;
use std::time::Instant;

// Use Lazy static initialization for global state, as Traefik doesn't restart our WASM plugin,
// we don't have to parse config & initialize HttpClient on every single request or response.
static CONFIG: Lazy<Config> = Lazy::new(Config::get_or_fallback);
static BLACKLIST: Lazy<RouteBlacklist> = Lazy::new(|| RouteBlacklist::new(&CONFIG.route_blacklist));

#[cfg(feature = "wasm")]
static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(|| HttpClient::new(CONFIG.treblle_api_urls.clone()));

/// The main handler for HTTP requests and responses
struct HttpHandler;

impl HttpHandler {
    /// Process an incoming HTTP request
    ///
    /// This function handles the incoming HTTP request, checks if it should be processed,
    /// and sends the relevant data to the Treblle API if necessary.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was processed successfully, or an error if something went wrong.
    #[cfg(feature = "wasm")]
    fn process_request(&self) -> Result<()> {
        let start_time = Instant::now();
        log(LogLevel::Info, "Processing request...");

        let uri = host_get_uri().map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get URI: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        log(LogLevel::Debug, &format!("Processing request for URI: {}", uri));

        if BLACKLIST.is_blacklisted(&uri) {
            log(LogLevel::Info, "URL is blacklisted, skipping Treblle API");
            return Ok(());
        }

        let content_type = host_get_header_values(0, "Content-Type").map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get Content-Type: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        log(LogLevel::Debug, &format!("Content-Type: {:?}", content_type));

        if !payload::is_json(&content_type) {
            log(LogLevel::Info, "Non-JSON request, skipping Treblle API");
            return Ok(());
        }

        let method = host_get_method().map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get HTTP method: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        let headers = self.get_headers(0).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get request headers: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        log(LogLevel::Debug, "Starting to read request body");

        let body = self.read_body(1).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to read request body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        log(LogLevel::Debug, "Creating Payload");

        let mut payload = Payload::new();

        log(LogLevel::Debug, "Updating request info in payload");
        payload.update_request_info(method, uri, headers, &body);
        payload.update_server_info(host_get_protocol_version().map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get protocol version: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?);
        payload.update_language_info();

        self.send_to_treblle(&payload, start_time)?;

        log(LogLevel::Info, "Request processing completed successfully");

        Ok(())
    }

    /// Process an HTTP response
    ///
    /// This function handles the HTTP response, checks if it should be processed,
    /// and sends the relevant data to the Treblle API if necessary.
    ///
    /// # Arguments
    ///
    /// * `_req_ctx` - The request context (unused)
    /// * `is_error` - Indicates if the response is an error
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the response was processed successfully, or an error if something went wrong.
    #[cfg(feature = "wasm")]
    fn process_response(&self, _req_ctx: i32, is_error: i32) -> Result<()> {
        if !CONFIG.buffer_response {
            log(LogLevel::Info, "Not processing response, buffer_response is not enabled");
            return Ok(());
        }

        let start_time = Instant::now();

        log(LogLevel::Info, "Processing response...");

        let mut payload = Payload::new();

        let headers = self.get_headers(1).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get response headers: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        let body = self.read_body(0).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to read response body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        let status_code = host_get_status_code();

        payload.update_response_info(status_code, headers, &body, start_time);
        payload.update_server_info(host_get_protocol_version().map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get protocol version: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?);
        payload.update_language_info();

        if is_error != 0 || status_code >= 400 {
            let error = ErrorInfo {
                source: "response".to_string(),
                error_type: "HTTP Error".to_string(),
                message: format!("HTTP status code: {}", status_code),
                file: String::new(),
                line: 0,
            };
            payload.add_error(error);
        }

        self.send_to_treblle(&payload, start_time)?;

        log(LogLevel::Info, "Response processing completed successfully");

        Ok(())
    }

    /// Get HTTP headers
    ///
    /// Retrieves the HTTP headers for either the request or response.
    ///
    /// # Arguments
    ///
    /// * `header_kind` - Specifies whether to get request (0) or response (1) headers
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `HashMap` of header names and values, or an error if retrieval fails.
    #[cfg(feature = "wasm")]
    fn get_headers(&self, header_kind: u32) -> Result<std::collections::HashMap<String, String>> {
        log(LogLevel::Info, "Starting get_headers");

        let mut headers = std::collections::HashMap::new();
        let header_names = host_get_header_names(header_kind).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get header names: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        for name in header_names.split(',').filter(|s| !s.is_empty()) {
            if let Ok(values) = host_get_header_values(header_kind, name) {
                headers.insert(name.to_string(), values);
            }
        }

        log(LogLevel::Info, &format!("Total headers processed: {}", headers.len()));

        Ok(headers)
    }

    /// Read the HTTP body
    ///
    /// Reads either the request or response body.
    ///
    /// # Arguments
    ///
    /// * `body_kind` - Specifies whether to read the request (0) or response (1) body
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the body as a vector of bytes, or an error if reading fails.
    #[cfg(feature = "wasm")]
    fn read_body(&self, body_kind: u32) -> Result<Vec<u8>> {
        host_read_body(body_kind).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to read body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })
    }

    /// Send data to Treblle API
    ///
    /// Sends the processed payload to the Treblle API.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload to send to Treblle API
    /// * `start_time` - The time when processing started
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the data was sent successfully, or an error if sending fails.
    #[cfg(feature = "wasm")]
    #[cfg(feature = "wasm")]
    fn send_to_treblle(&self, payload: &Payload, start_time: Instant) -> Result<()> {
        log(LogLevel::Info, "Preparing to send data to Treblle API");

        let payload_json = payload.to_json()?;

        log(LogLevel::Info, &format!("Payload JSON length: {}", payload_json.len()));

        HTTP_CLIENT
            .post(payload_json.as_bytes(), &CONFIG.api_key)
            .map_err(|e| {
                log(LogLevel::Error, &format!("Failed to send data to Treblle API: {}", e));
                TreblleError::Http(format!("Failed to send data to Treblle API: {}", e))
            })?;

        log(
            LogLevel::Info,
            &format!(
                "Data sent successfully to Treblle API in {} ms",
                start_time.elapsed().as_millis()
            ),
        );

        Ok(())
    }
}

#[cfg(feature = "wasm")]
impl Guest for HttpHandler {
    /// Handle an incoming HTTP request
    ///
    /// This function is called by the Traefik middleware to process an incoming HTTP request.
    ///
    /// # Returns
    ///
    /// Returns 1 to indicate that Traefik should continue processing the request.
    fn handle_request() -> i64 {
        logger::init();
        log(LogLevel::Info, "Handling request in WASM module");

        let handler = HttpHandler;

        log(LogLevel::Info, &format!("Buffer response is set to: {}", CONFIG.buffer_response));

        if CONFIG.buffer_response {
            let features = host_enable_features(2);  // Enable FeatureBufferResponse
            log(LogLevel::Info, &format!("Enabled features: {}", features));
        }

        if let Err(e) = handler.process_request() {
            log(LogLevel::Error, &format!("Error processing request: {}", e));
        }

        log(LogLevel::Info, "Letting Traefik continue processing the request");

        1 // Always continue processing the request
    }

    /// Handle an HTTP response
    ///
    /// This function is called by the Traefik middleware to process an HTTP response.
    ///
    /// # Arguments
    ///
    /// * `req_ctx` - The request context
    /// * `is_error` - Indicates if the response is an error
    fn handle_response(req_ctx: i32, is_error: i32) {
        logger::init();
        log(LogLevel::Info, "Handling response in WASM module");

        let handler = HttpHandler;

        if let Err(e) = handler.process_response(req_ctx, is_error) {
            log(LogLevel::Error, &format!("Error processing response: {}", e));
        }

        log(LogLevel::Info, "Finished processing response");
    }
}

#[cfg(feature = "wasm")]
#[no_mangle]
pub extern "C" fn handle_request() -> i64 {
    HttpHandler::handle_request()
}

#[cfg(feature = "wasm")]
#[no_mangle]
pub extern "C" fn handle_response(req_ctx: i32, is_error: i32) {
    HttpHandler::handle_response(req_ctx, is_error)
}