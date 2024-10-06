//! Treblle middleware for Traefik
//!
//! This module implements a WASM-based middleware for Traefik that integrates
//! with Treblle's API monitoring and logging services.

#![cfg_attr(test, allow(unused_imports, dead_code))]

#[cfg(feature = "wasm")]
mod bindings;
#[cfg(feature = "wasm")]
mod host_functions;

mod certs;
mod config;
mod constants;
mod error;
mod http_handler;
mod logger;
mod payload;
mod route_blacklist;
mod schema;
mod utils;
mod wasi_http_client;

use once_cell::sync::Lazy;

#[cfg(feature = "wasm")]
use bindings::exports::traefik::http_handler::handler::Guest;

use config::Config;
use http_handler::HttpHandler;
use logger::{log, LogLevel};
use route_blacklist::RouteBlacklist;
use crate::wasi_http_client::WasiHttpClient;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::get_or_fallback);
pub static BLACKLIST: Lazy<RouteBlacklist> =
    Lazy::new(|| RouteBlacklist::new(&CONFIG.route_blacklist));

#[cfg(feature = "wasm")]
pub static HTTP_CLIENT: Lazy<WasiHttpClient> = Lazy::new(|| {
    WasiHttpClient::new(CONFIG.treblle_api_urls.clone())
        .expect("Failed to initialize WasiHttpClient")
});

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

        Lazy::force(&HTTP_CLIENT);
        
        log(LogLevel::Debug, "Initializing request handler!");
        log(LogLevel::Info, "Handling request in WASM module");

        log(
            LogLevel::Info,
            &format!("Buffer response is set to: {}", CONFIG.buffer_response),
        );

        if CONFIG.buffer_response {
            let features = host_functions::host_enable_features(2); // Enable FeatureBufferResponse
            log(LogLevel::Info, &format!("Enabled features: {}", features));
        }

        if let Err(e) = HttpHandler.process_request() {
            log(LogLevel::Error, &format!("Error processing request: {}", e));
        }

        log(
            LogLevel::Info,
            "Letting Traefik continue processing the request with next middleware",
        );

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

        log(LogLevel::Debug, "Initializing response handler");
        log(LogLevel::Info, "Handling response in WASM module");

        if let Err(e) = HttpHandler.process_response(req_ctx, is_error) {
            log(
                LogLevel::Error,
                &format!("Error processing response: {}", e),
            );
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
