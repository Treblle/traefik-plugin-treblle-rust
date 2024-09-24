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
use constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use error::{Result, TreblleError};
use http_client::HttpClient;
use payload::Payload;
use route_blacklist::RouteBlacklist;
use schema::ErrorInfo;
use std::time::Instant;

// Use Lazy static initialization for global state
static CONFIG: Lazy<Config> = Lazy::new(Config::get_or_fallback);
static BLACKLIST: Lazy<RouteBlacklist> = Lazy::new(|| RouteBlacklist::new(&CONFIG.route_blacklist));

#[cfg(feature = "wasm")]
static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(|| HttpClient::new(CONFIG.treblle_api_urls.clone()));

/// The main handler for HTTP requests and responses
struct HttpHandler;

impl HttpHandler {
    /// Process an incoming HTTP request
    #[cfg(feature = "wasm")]
    fn process_request(&self) -> Result<()> {
        let start_time = Instant::now();
        
        host_log(LOG_LEVEL_INFO, "Starting process_request");

        let uri = host_get_uri()?;
        
        host_log(LOG_LEVEL_INFO, &format!("Processing request for URI: {}", uri));

        if BLACKLIST.is_blacklisted(&uri) {
            host_log(LOG_LEVEL_INFO, "URL is blacklisted, skipping Treblle API processing");
            
            return Ok(());
        }

        let content_type = host_get_header_values(0, "Content-Type")
            .map_err(|e| {
                host_log(LOG_LEVEL_ERROR, &format!("Failed to get Content-Type: {}", e));
                TreblleError::HostFunction(e.to_string())
            })?;
        
        host_log(LOG_LEVEL_INFO, &format!("Content-Type: {:?}", content_type));

        if !payload::is_json(&content_type) {
            host_log(LOG_LEVEL_INFO, "Non-JSON request, skipping Treblle API");
            
            return Ok(());
        }

        let method = host_get_method()?;

        let headers = self.get_headers(0).map_err(|e| {
            host_log(LOG_LEVEL_ERROR, &format!("Failed to get request headers: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        host_log(LOG_LEVEL_INFO, "Starting to read request body");

        let body = self.read_body(1).map_err(|e| {
            host_log(LOG_LEVEL_ERROR, &format!("Failed to read request body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        host_log(LOG_LEVEL_INFO, "Creating Payload");
        
        let mut payload = Payload::new();

        host_log(LOG_LEVEL_INFO, "Updating request info in payload");
        payload.update_request_info(method, uri, headers, &body);
        payload.update_server_info(host_get_protocol_version()?);
        payload.update_language_info();

        self.send_to_treblle(&payload, start_time)?;

        host_log(LOG_LEVEL_INFO, "Request processing completed successfully");

        Ok(())
    }

    /// Process an HTTP response
    #[cfg(feature = "wasm")]
    fn process_response(&self, _req_ctx: i32, is_error: i32) -> Result<()> {
        if !CONFIG.buffer_response {
            host_log(LOG_LEVEL_INFO, "Not processing response, buffer_response is not enabled");
            
            return Ok(());
        }

        let start_time = Instant::now();
        
        host_log(LOG_LEVEL_INFO, "Starting process_response");

        let mut payload = Payload::new();

        let headers = self.get_headers(1).map_err(|e| {
            host_log(LOG_LEVEL_ERROR, &format!("Failed to get response headers: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;

        let body = self.read_body(0).map_err(|e| {
            host_log(LOG_LEVEL_ERROR, &format!("Failed to read response body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })?;
        
        let status_code = host_get_status_code();

        payload.update_response_info(status_code, headers, &body, start_time);
        payload.update_server_info(host_get_protocol_version()?);
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

        host_log(LOG_LEVEL_INFO, "Response processing completed successfully");

        Ok(())
    }

    /// Get HTTP headers
    #[cfg(feature = "wasm")]
    fn get_headers(&self, header_kind: u32) -> Result<std::collections::HashMap<String, String>> {
        host_log(LOG_LEVEL_INFO, "Starting get_headers");

        let mut headers = std::collections::HashMap::new();
        let header_names = host_get_header_names(header_kind)?;

        for name in header_names.split(',').filter(|s| !s.is_empty()) {
            if let Ok(values) = host_get_header_values(header_kind, name) {
                headers.insert(name.to_string(), values);
            }
        }

        host_log(LOG_LEVEL_INFO, &format!("Total headers processed: {}", headers.len()));

        Ok(headers)
    }

    /// Read the HTTP body
    #[cfg(feature = "wasm")]
    fn read_body(&self, body_kind: u32) -> Result<Vec<u8>> {
        match host_read_body(body_kind) {
            Ok(body) => {
                Ok(body)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    /// Send data to Treblle API
    #[cfg(feature = "wasm")]
    fn send_to_treblle(&self, payload: &Payload, start_time: Instant) -> Result<()> {
        host_log(LOG_LEVEL_INFO, "Preparing to send data to Treblle API");

        let payload_json = payload.to_json()?;
        host_log(LOG_LEVEL_INFO, &format!("Payload JSON length: {}", payload_json.len()));

        HTTP_CLIENT
            .post(payload_json.as_bytes(), &CONFIG.api_key)
            .map_err(|e| TreblleError::Http(format!("Failed to send data to Treblle API: {}", e)))?;

        host_log(
            LOG_LEVEL_INFO,
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
    fn handle_request() -> i64 {
        host_log(LOG_LEVEL_INFO, "Handling request in WASM module");

        let handler = HttpHandler;

        host_log(LOG_LEVEL_INFO, &format!("Buffer response is set to: {}", CONFIG.buffer_response));

        if CONFIG.buffer_response {
            let features = host_enable_features(2);  // Enable FeatureBufferResponse
            host_log(LOG_LEVEL_INFO, &format!("Enabled features: {}", features));
        }

        if let Err(e) = handler.process_request() {
            host_log(LOG_LEVEL_ERROR, &format!("Error processing request: {}", e));
        }

        host_log(LOG_LEVEL_INFO, "Letting Traefik continue processing the request");

        1 // Always continue processing the request
    }

    fn handle_response(req_ctx: i32, is_error: i32) {
        host_log(LOG_LEVEL_INFO, "Handling response in WASM module");

        let handler = HttpHandler;

        if let Err(e) = handler.process_response(req_ctx, is_error) {
            host_log(LOG_LEVEL_ERROR, &format!("Error processing response: {}", e));
        }

        host_log(LOG_LEVEL_INFO, "Finished processing response");
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