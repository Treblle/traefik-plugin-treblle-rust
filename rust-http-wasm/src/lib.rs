mod bindings;
mod config;
mod constants;
mod host_functions;
mod http_client;
mod payload;
mod route_blacklist;
mod schema;

use bindings::exports::traefik::http_handler::handler::Guest;
use config::Config;
use http_client::HttpClient;
use payload::{is_json, Payload};
use route_blacklist::RouteBlacklist;

use crate::constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};

use crate::host_functions::{
    host_get_header_names, host_get_header_values, host_get_method, host_get_source_addr,
    host_get_uri, host_log, host_read_request_body, host_write_request_body,
};

struct HttpHandler;

impl Guest for HttpHandler {
    fn handle_request() -> i64 {
        host_log(LOG_LEVEL_INFO, "Handling request in WASM module");

        let config = Config::get_or_fallback();
        let blacklist = RouteBlacklist::new(&config.route_blacklist);

        let uri = host_get_uri().unwrap_or_else(|_| "Unknown".to_string());

        if blacklist.is_blacklisted(&uri) {
            host_log(
                LOG_LEVEL_INFO,
                "URL is blacklisted, skipping Treblle API processing",
            );
            return 1; // Continue processing the request
        }

        // Check if the request is JSON
        let content_type =
            host_get_header_values(0, "Content-Type").unwrap_or_else(|_| "".to_string());

        if is_json(&content_type) {
            // Read the request body
            let body = host_read_request_body().unwrap_or_else(|_| "{}".to_string());

            // Process the request for Treblle API
            let mut payload = Payload::new(&config);

            // Populate payload with request details
            let method = host_get_method().unwrap_or_else(|_| "Unknown".to_string());
            let ip = host_get_source_addr().unwrap_or_else(|_| "Unknown".to_string());

            // Populate headers
            let mut headers = std::collections::HashMap::new();
            let header_names = host_get_header_names(0).unwrap_or_else(|_| "[]".to_string());
            let header_names: Vec<String> =
                serde_json::from_str(&header_names).unwrap_or_else(|_| vec![]);
            for name in header_names {
                let values = host_get_header_values(0, &name).unwrap_or_else(|_| "[]".to_string());
                let values: Vec<String> = serde_json::from_str(&values).unwrap_or_else(|_| vec![]);
                headers.insert(name, values.join(", "));
            }

            payload.update_request_info(method, uri, ip, headers, body.clone());
            payload.mask_sensitive_data();

            let client = HttpClient::new();
            let payload_json = payload.to_json();
            match client.send_to_treblle(
                &config.treblle_api_url,
                payload_json.as_bytes(),
                &config.api_key,
            ) {
                Ok(_) => host_log(LOG_LEVEL_INFO, "Data sent to Treblle API"),
                Err(e) => host_log(
                    LOG_LEVEL_ERROR,
                    &format!("Error sending data to Treblle API: {}", e),
                ),
            }

            // Set the request body back
            if let Err(e) = host_write_request_body(body.as_bytes()) {
                host_log(
                    LOG_LEVEL_ERROR,
                    &format!("Error setting request body back: {}", e),
                );
            }
        } else {
            host_log(LOG_LEVEL_INFO, "Non-JSON request, skipping Treblle API");
        }

        host_log(
            LOG_LEVEL_INFO,
            "Letting Traefik continue processing the request",
        );
        1 // Always continue processing the request, regardless of Treblle API processing
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
