mod bindings;
mod config;
mod constants;
mod host_functions;
mod http_client;
mod payload;
mod route_blacklist;
mod schema;

use anyhow::Result;
use bindings::exports::traefik::http_handler::handler::Guest;
use config::Config;
use http_client::HttpClient;
use payload::{is_json, Payload};
use route_blacklist::RouteBlacklist;

use crate::constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::host_functions::*;

struct HttpHandler;

impl HttpHandler {
    fn process_request(config: &Config, blacklist: &RouteBlacklist) -> Result<()> {
        host_log(LOG_LEVEL_INFO, "Starting process_request");

        let uri = host_get_uri().map_err(|e| anyhow::anyhow!("Failed to get URI: {}", e))?;
        host_log(
            LOG_LEVEL_INFO,
            &format!("Processing request for URI: {}", uri),
        );

        if blacklist.is_blacklisted(&uri) {
            host_log(
                LOG_LEVEL_INFO,
                "URL is blacklisted, skipping Treblle API processing",
            );
            return Ok(());
        }

        let content_type = host_get_header_values(0, "Content-Type")
            .map_err(|e| anyhow::anyhow!("Failed to get Content-Type: {}", e))?;
        host_log(LOG_LEVEL_INFO, &format!("Content-Type: {}", content_type));

        if !is_json(&content_type) {
            host_log(LOG_LEVEL_INFO, "Non-JSON request, skipping Treblle API");
            return Ok(());
        }

        host_log(LOG_LEVEL_INFO, "Starting to read request body");
        let body = match host_read_request_body() {
            Ok(body) => {
                host_log(
                    LOG_LEVEL_INFO,
                    &format!("Successfully read body: {} bytes", body.len()),
                );
                body
            }
            Err(e) => {
                host_log(
                    LOG_LEVEL_ERROR,
                    &format!("Failed to read request body: {}", e),
                );
                return Err(anyhow::anyhow!("Failed to read request body: {}", e));
            }
        };

        // Immediately write the body back
        host_log(LOG_LEVEL_INFO, "Writing request body back");
        host_write_request_body(body.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write request body back: {}", e))?;

        host_log(
            LOG_LEVEL_INFO,
            &format!("Request body length: {}", body.len()),
        );

        let mut payload = Payload::new(config);
        host_log(LOG_LEVEL_INFO, "Populating payload");

        // Get headers, but continue even if it fails
        let headers = match Self::get_headers() {
            Ok(h) => h,
            Err(e) => {
                host_log(
                    LOG_LEVEL_ERROR,
                    &format!(
                        "Failed to get headers: {}. Continuing with empty headers.",
                        e
                    ),
                );
                std::collections::HashMap::new()
            }
        };

        // Populate payload with available information
        let method = host_get_method().unwrap_or_else(|_| "Unknown".to_string());
        let ip = host_get_source_addr().unwrap_or_else(|_| "Unknown".to_string());
        payload.update_request_info(method, uri, ip, headers, body);

        host_log(LOG_LEVEL_INFO, "Masking sensitive data");
        payload.mask_sensitive_data();

        host_log(LOG_LEVEL_INFO, "Sending to Treblle");
        Self::send_to_treblle(config, &payload)?;

        host_log(LOG_LEVEL_INFO, "Request processing completed successfully");
        Ok(())
    }

    fn populate_payload(payload: &mut Payload, uri: &str, body: &str) -> Result<()> {
        host_log(LOG_LEVEL_INFO, "Starting populate_payload");

        let method =
            host_get_method().map_err(|e| anyhow::anyhow!("Failed to get method: {}", e))?;
        host_log(LOG_LEVEL_INFO, &format!("Method: {}", method));

        let ip = host_get_source_addr()
            .map_err(|e| anyhow::anyhow!("Failed to get source address: {}", e))?;
        host_log(LOG_LEVEL_INFO, &format!("Source IP: {}", ip));

        let headers =
            Self::get_headers().map_err(|e| anyhow::anyhow!("Failed to get headers: {}", e))?;
        host_log(
            LOG_LEVEL_INFO,
            &format!("Number of headers: {}", headers.len()),
        );

        payload.update_request_info(method, uri.to_string(), ip, headers, body.to_string());
        host_log(LOG_LEVEL_INFO, "Payload populated successfully");
        Ok(())
    }

    fn get_headers() -> Result<std::collections::HashMap<String, String>> {
        host_log(LOG_LEVEL_INFO, "Starting get_headers");
        let mut headers = std::collections::HashMap::new();

        let header_names_str = host_get_header_names(0)?;
        host_log(
            LOG_LEVEL_INFO,
            &format!("Raw header names: {}", header_names_str),
        );

        // Split the header names string into individual names
        let header_names: Vec<String> = header_names_str
            .split(|c: char| !c.is_ascii_alphabetic() && c != '-')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        host_log(
            LOG_LEVEL_INFO,
            &format!("Number of header names: {}", header_names.len()),
        );

        for name in header_names {
            let values = host_get_header_values(0, &name)?;
            // Remove null characters from the header value
            let cleaned_values = values.trim_end_matches('\0').to_string();
            host_log(
                LOG_LEVEL_INFO,
                &format!("Header '{}': {}", name, cleaned_values),
            );
            headers.insert(name, cleaned_values);
        }

        host_log(
            LOG_LEVEL_INFO,
            &format!("Total headers processed: {}", headers.len()),
        );
        Ok(headers)
    }

    fn send_to_treblle(config: &Config, payload: &Payload) -> Result<()> {
        host_log(LOG_LEVEL_INFO, "Preparing to send data to Treblle API");
        let client = HttpClient::new();

        host_log(LOG_LEVEL_INFO, "Converting payload to JSON");
        let payload_json = payload.to_json()?;

        host_log(
            LOG_LEVEL_INFO,
            &format!("Payload JSON length: {}", payload_json.len()),
        );
        host_log(LOG_LEVEL_INFO, &format!("Payload JSON: {}", payload_json));

        host_log(
            LOG_LEVEL_INFO,
            &format!("Sending request to Treblle API: {}", config.treblle_api_url),
        );
        match client.send_to_treblle(
            &config.treblle_api_url,
            payload_json.as_bytes(),
            &config.api_key,
        ) {
            Ok(_) => {
                host_log(LOG_LEVEL_INFO, "Data sent successfully to Treblle API");
                Ok(())
            }
            Err(e) => {
                host_log(
                    LOG_LEVEL_ERROR,
                    &format!("Error sending data to Treblle API: {}", e),
                );
                Err(anyhow::anyhow!("Failed to send data to Treblle API: {}", e))
            }
        }
    }
}

impl Guest for HttpHandler {
    fn handle_request() -> i64 {
        host_log(LOG_LEVEL_INFO, "Handling request in WASM module");

        let config = Config::get_or_fallback();
        let blacklist = RouteBlacklist::new(&config.route_blacklist);

        if let Err(e) = Self::process_request(&config, &blacklist) {
            host_log(LOG_LEVEL_ERROR, &format!("Error processing request: {}", e));
        }

        host_log(
            LOG_LEVEL_INFO,
            "Letting Traefik continue processing the request",
        );
        1 // Always continue processing the request
    }

    fn handle_response(_req_ctx: i32, _is_error: i32) {
        // This function is called after the response is generated.
        // We don't need to modify anything here for now.
    }
}

#[no_mangle]
pub extern "C" fn handle_request() -> i64 {
    HttpHandler::handle_request()
}

#[no_mangle]
pub extern "C" fn handle_response(req_ctx: i32, is_error: i32) {
    HttpHandler::handle_response(req_ctx, is_error)
}
