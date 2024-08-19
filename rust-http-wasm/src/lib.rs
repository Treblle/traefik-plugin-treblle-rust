mod host_functions;

use crate::exports::traefik::http_handler::handler::Guest;
use once_cell::sync::Lazy;
use reqwest;
use serde_json::{json, Value};
use std::sync::Mutex;

use serde::Deserialize;

use host_functions::{log_message, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_WARN};

wit_bindgen::generate!({
    path: "traefik-http-handler.wit",
    world: "http-handler",
    exports: {
        "traefik:http-handler/handler": HttpHandler,
    },
});

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

#[derive(Deserialize, serde::Serialize)]
struct Config {
    treblle_api_url: String,
}

struct HttpHandler;

impl Guest for HttpHandler {
    fn set_config(config_str: String) {
        log_message(
            LOG_LEVEL_INFO,
            &format!("Received raw config: {}", config_str),
        );

        match serde_json::from_str::<Value>(&config_str) {
            Ok(value) => {
                log_message(LOG_LEVEL_INFO, &format!("Parsed config: {:?}", value));
                if let Some(url) = value.get("treblleApiUrl").and_then(|v| v.as_str()) {
                    let config = Config {
                        treblle_api_url: url.to_string(),
                    };
                    let mut cfg = CONFIG.lock().unwrap();
                    *cfg = Some(config);
                    log_message(LOG_LEVEL_INFO, &format!("Config set: {}", url));
                } else {
                    log_message(LOG_LEVEL_ERROR, "treblleApiUrl not found in config");
                }
            }
            Err(e) => {
                log_message(LOG_LEVEL_ERROR, &format!("Failed to parse config: {}", e));
            }
        }
    }

    fn handle_request() -> i64 {
        log_message(LOG_LEVEL_INFO, "Handling request in WASM module");

        let config = CONFIG.lock().unwrap();

        let treblle_api_url = if let Some(cfg) = config.as_ref() {
            cfg.treblle_api_url.clone()
        } else {
            log_message(LOG_LEVEL_WARN, "Config not set, using fallback URL");
            "http://treblle-api:3002/api".to_string()
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Make an asynchronous HTTP request to Treblle API
        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            let payload = json!({
                "message": "Request processed by Treblle Middleware",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            log_message(
                LOG_LEVEL_INFO,
                &format!("Sending request to Treblle API: {}", treblle_api_url),
            );

            client.post(treblle_api_url).json(&payload).send().await
        });

        match result {
            Ok(response) => {
                log_message(
                    LOG_LEVEL_INFO,
                    &format!("Sent data to Treblle API. Status: {}", response.status()),
                );
                0 // Success
            }
            Err(e) => {
                log_message(
                    LOG_LEVEL_ERROR,
                    &format!("Error sending data to Treblle API: {}", e),
                );
                1 // Error
            }
        }
    }

    fn handle_response(req_ctx: i32, is_error: i32) {
        log_message(
            LOG_LEVEL_INFO,
            &format!(
                "Handling response in WASM module. req_ctx: {}, is_error: {}",
                req_ctx, is_error
            ),
        );
    }
}

// Explicitly export the functions so Traefik can call them, seems like a limitation of wit-bindgen for now
#[no_mangle]
pub extern "C" fn handle_request() -> i64 {
    HttpHandler::handle_request()
}

#[no_mangle]
pub extern "C" fn handle_response(req_ctx: i32, is_error: i32) {
    HttpHandler::handle_response(req_ctx, is_error)
}

#[no_mangle]
pub extern "C" fn set_config(ptr: *mut u8, len: usize) {
    let config_str = unsafe { String::from_raw_parts(ptr, len, len) };
    HttpHandler::set_config(config_str);
}
