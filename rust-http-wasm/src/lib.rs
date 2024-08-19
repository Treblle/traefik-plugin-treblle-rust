mod bindings;
mod host_functions;

use bindings::exports::traefik::http_handler::handler::Guest;
use serde::Deserialize;
use serde_json::{json, Value};
use wasmedge_http_req::request;

use host_functions::{middleware_get_config, middleware_log, LOG_LEVEL_ERROR, LOG_LEVEL_INFO};

#[derive(Deserialize, serde::Serialize, Clone)]
struct Config {
    treblle_api_url: String,
}

struct HttpHandler;

impl Guest for HttpHandler {
    fn handle_request() -> i64 {
        middleware_log(LOG_LEVEL_INFO, "Handling request in WASM module");

        let config = get_config_or_fallback();

        // Test random public service (using HTTP instead of HTTPS)
        let public_test_result = match perform_http_get("http://httpbin.org/ip") {
            Ok(response) => format!("Public test success, {}", response),
            Err(e) => format!("Public test failed: {}", e),
        };

        middleware_log(LOG_LEVEL_INFO, &public_test_result);

        // Make an HTTP request to mocked Treblle API
        let payload = json!({
            "message": "Request processed by Treblle Middleware",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let treblle_result =
            match perform_http_post(&config.treblle_api_url, payload.to_string().as_bytes()) {
                Ok(response) => format!("Data sent to Treblle API, {}", response),
                Err(e) => format!("Error sending data to Treblle API: {}", e),
            };

        middleware_log(LOG_LEVEL_INFO, &treblle_result);

        if treblle_result.contains("Error") {
            1 // Error
        } else {
            0 // Success
        }
    }

    fn handle_response(req_ctx: i32, is_error: i32) {
        middleware_log(
            LOG_LEVEL_INFO,
            &format!(
                "Handling response in WASM module. req_ctx: {}, is_error: {}",
                req_ctx, is_error
            ),
        );
    }
}

fn perform_http_get(url: &str) -> Result<String, String> {
    middleware_log(
        LOG_LEVEL_INFO,
        &format!("Performing HTTP GET request to {}", url),
    );

    let mut writer = Vec::new();
    let response = match request::get(url, &mut writer) {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to send GET request: {}", e)),
    };

    if !response.status_code().is_success() {
        return Err(format!("HTTP error: {}", response.status_code()));
    }

    String::from_utf8(writer).map_err(|e| format!("Failed to decode response: {}", e))
}

fn perform_http_post(url: &str, body: &[u8]) -> Result<String, String> {
    middleware_log(
        LOG_LEVEL_INFO,
        &format!(
            "Performing HTTP POST request to {} with payload: {}",
            url,
            String::from_utf8_lossy(body)
        ),
    );

    let mut writer = Vec::new();
    let response = match request::post(url, body, &mut writer) {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to send POST request: {}", e)),
    };

    if !response.status_code().is_success() {
        return Err(format!("HTTP error: {}", response.status_code()));
    }

    String::from_utf8(writer).map_err(|e| format!("Failed to decode response: {}", e))
}

fn get_config_or_fallback() -> Config {
    let raw_config = middleware_get_config();
    middleware_log(LOG_LEVEL_INFO, &format!("Raw config: {}", raw_config));

    match serde_json::from_str::<Value>(&raw_config) {
        Ok(value) => {
            if let Some(url) = value.get("treblleApiUrl").and_then(|v| v.as_str()) {
                Config {
                    treblle_api_url: url.to_string(),
                }
            } else {
                middleware_log(
                    LOG_LEVEL_ERROR,
                    "treblleApiUrl not found in config, using fallback",
                );
                Config {
                    treblle_api_url: "http://treblle-api:3002/api".to_string(),
                }
            }
        }
        Err(e) => {
            middleware_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to parse config: {}, using fallback", e),
            );
            Config {
                treblle_api_url: "http://treblle-api:3002/api".to_string(),
            }
        }
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
