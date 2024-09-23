mod bindings;
mod config;
mod constants;
mod error;
mod host_functions;
mod http_client;
mod payload;
mod route_blacklist;
mod schema;

use bindings::exports::traefik::http_handler::handler::Guest;
use config::Config;
use error::{Result, TreblleError};
use http_client::HttpClient;
use payload::{is_json, Payload};
use route_blacklist::RouteBlacklist;

use crate::constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::host_functions::*;

struct HttpHandler;

impl HttpHandler {
    fn process_request(config: &Config, blacklist: &RouteBlacklist) -> Result<()> {
        host_log(LOG_LEVEL_INFO, "Starting process_request");

        let uri = host_get_uri().map_err(|e| {
            host_log(LOG_LEVEL_ERROR, &format!("Failed to get URI: {}", e));

            TreblleError::HostFunction(format!("Failed to get URI: {}", e))
        })?;

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

        let content_type = host_get_header_values(0, "Content-Type").map_err(|e| {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to get Content-Type: {}", e),
            );
            TreblleError::HostFunction(format!("Failed to get Content-Type: {}", e))
        })?;

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
                return Err(TreblleError::HostFunction(format!(
                    "Failed to read request body: {}",
                    e
                )));
            }
        };

        host_log(LOG_LEVEL_INFO, "Writing request body back");

        if let Err(e) = host_write_request_body(body.as_bytes()) {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to write request body back: {}", e),
            );
            return Err(TreblleError::HostFunction(format!(
                "Failed to write request body back: {}",
                e
            )));
        }

        host_log(
            LOG_LEVEL_INFO,
            &format!("Request body length: {}", body.len()),
        );

        host_log(LOG_LEVEL_INFO, "Creating Payload");

        let mut payload = Payload::new(config);

        host_log(LOG_LEVEL_INFO, "Populating payload");
        host_log(LOG_LEVEL_INFO, "Getting headers");

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

        host_log(LOG_LEVEL_INFO, "Getting method");

        let method = host_get_method().unwrap_or_else(|e| {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to get method: {}. Using 'Unknown'.", e),
            );
            "Unknown".to_string()
        });

        host_log(LOG_LEVEL_INFO, "Getting source address");

        let ip = host_get_source_addr().unwrap_or_else(|e| {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to get source address: {}. Using 'Unknown'.", e),
            );

            "Unknown".to_string()
        });

        host_log(LOG_LEVEL_INFO, "Updating request info in payload");

        payload.update_request_info(method, uri, ip, headers, body);

        host_log(LOG_LEVEL_INFO, "Masking sensitive data");

        payload.mask_sensitive_data();

        host_log(LOG_LEVEL_INFO, "Sending to Treblle");

        if let Err(e) = Self::send_to_treblle(config, &payload) {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Error sending data to Treblle API: {}", e),
            );

            return Err(e);
        }

        host_log(LOG_LEVEL_INFO, "Request processing completed successfully");

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

        client
            .post(
                &config.treblle_api_url,
                payload_json.as_bytes(),
                &config.api_key,
            )
            .map_err(|e| {
                TreblleError::Http(format!("Failed to send data to Treblle API: {}", e))
            })?;

        host_log(LOG_LEVEL_INFO, "Data sent successfully to Treblle API");

        Ok(())
    }

    fn verify_https_support() -> Result<()> {
        let client = HttpClient::new();
        let url = "https://example.com";

        host_log(
            LOG_LEVEL_INFO,
            &format!("Verifying HTTPS support by calling: {}", url),
        );

        let response = client
            .get(url)
            .map_err(|e| TreblleError::Http(format!("Failed to send GET request: {}", e)))?;

        if response.status_code().is_success() {
            host_log(LOG_LEVEL_INFO, "Successfully verified HTTPS support");
            Ok(())
        } else {
            let error_msg = format!(
                "HTTPS verification failed with status code: {}",
                response.status_code()
            );
            host_log(LOG_LEVEL_ERROR, &error_msg);
            Err(TreblleError::Http(error_msg))
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

        if let Err(e) = Self::verify_https_support() {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("HTTPS verification failed: {}", e),
            );
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
