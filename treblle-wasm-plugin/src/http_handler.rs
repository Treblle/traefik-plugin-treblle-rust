//! HTTP Handler for Treblle middleware
//!
//! This module contains the main logic for processing HTTP requests and responses
//! before sending them to the Treblle API.

use std::collections::HashMap;
use std::time::Instant;
use std::sync::MutexGuard;

#[cfg(feature = "wasm")]
use crate::host_functions::*;

#[cfg(feature = "wasm")]
use crate::{HTTP_CLIENT};

use crate::constants::{HEADER_CONTENT_TYPE, REQUEST_KIND, RESPONSE_KIND};
use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};
use crate::payload::Payload;
use crate::schema::ErrorInfo;
use crate::{BLACKLIST, CONFIG};

/// The main handler for HTTP requests and responses
pub struct HttpHandler;

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
    pub fn process_request(&self) -> Result<()> {
        let start_time = Instant::now();
        log(LogLevel::Debug, "Processing request...");

        let uri = self.get_uri()?;
        if BLACKLIST.is_blacklisted(&uri) {
            log(LogLevel::Info, "URL is blacklisted, skipping Treblle API");
            return Ok(());
        }

        let content_type = self.get_content_type()?;
        if !crate::payload::is_json(&content_type) {
            log(LogLevel::Info, "Non-JSON request, skipping Treblle API");
            return Ok(());
        }

        let method = self.get_method()?;
        let headers = self.get_headers(REQUEST_KIND)?;
        let body = self.read_body(REQUEST_KIND)?;

        self.write_body(REQUEST_KIND, &body)?;

        let mut payload = Payload::new();
        self.update_payload(&mut payload, method, uri, headers, &body)?;

        self.send_to_treblle(&payload, start_time)?;

        log(LogLevel::Debug, "Request processing completed successfully");

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
    pub fn process_response(&self, _req_ctx: i32, is_error: i32) -> Result<()> {
        if !CONFIG.buffer_response {
            log(
                LogLevel::Info,
                "Not processing response, buffer_response is not enabled",
            );
            return Ok(());
        }

        let start_time = Instant::now();

        log(LogLevel::Debug, "Processing response...");

        let mut payload = Payload::new();
        let headers = self.get_headers(RESPONSE_KIND)?;
        let body = self.read_body(RESPONSE_KIND)?;
        let status_code = host_get_status_code();

        self.write_body(RESPONSE_KIND, &body)?;

        payload.update_response_info(status_code, headers, &body, start_time);

        self.update_payload_server_info(&mut payload)?;

        if is_error != 0 || status_code >= 400 {
            payload.add_error(self.create_error_info(status_code));
        }

        self.send_to_treblle(&payload, start_time)?;

        log(LogLevel::Debug, "Response processing completed successfully");

        Ok(())
    }

    #[cfg(feature = "wasm")]
    fn get_uri(&self) -> Result<String> {
        host_get_uri().map_err(|e| {
            log(LogLevel::Error, &format!("Failed to get URI: {}", e));
            TreblleError::HostFunction(e.to_string())
        })
    }

    #[cfg(feature = "wasm")]
    fn get_content_type(&self) -> Result<String> {
        host_get_header_values(REQUEST_KIND, HEADER_CONTENT_TYPE).map_err(|e| {
            log(
                LogLevel::Error,
                &format!("Failed to get Content-Type: {}", e),
            );
            TreblleError::HostFunction(e.to_string())
        })
    }

    #[cfg(feature = "wasm")]
    fn get_method(&self) -> Result<String> {
        host_get_method().map_err(|e| {
            log(
                LogLevel::Error,
                &format!("Failed to get HTTP method: {}", e),
            );
            TreblleError::HostFunction(e.to_string())
        })
    }

    #[cfg(feature = "wasm")]
    fn get_headers(&self, header_kind: u32) -> Result<HashMap<String, String>> {
        log(LogLevel::Debug, "Starting get_headers");

        let header_names = host_get_header_names(header_kind).map_err(|e| {
            log(
                LogLevel::Error,
                &format!("Failed to get header names: {}", e),
            );
            TreblleError::HostFunction(e.to_string())
        })?;

        let mut headers = HashMap::new();

        for name in header_names.split(',').filter(|s| !s.is_empty()) {
            if let Ok(values) = host_get_header_values(header_kind, name) {
                headers.insert(name.to_string(), values);
            }
        }

        log(
            LogLevel::Debug,
            &format!("Total headers processed: {}", headers.len()),
        );

        Ok(headers)
    }

    #[cfg(feature = "wasm")]
    fn read_body(&self, body_kind: u32) -> Result<Vec<u8>> {
        host_read_body(body_kind).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to read body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })
    }

    #[cfg(feature = "wasm")]
    fn write_body(&self, body_kind: u32, body: &[u8]) -> Result<()> {
        host_write_body(body_kind, body).map_err(|e| {
            log(LogLevel::Error, &format!("Failed to write body: {}", e));
            TreblleError::HostFunction(e.to_string())
        })
    }

    #[cfg(feature = "wasm")]
    fn update_payload(
        &self,
        payload: &mut Payload,
        method: String,
        uri: String,
        headers: HashMap<String, String>,
        body: &[u8],
    ) -> Result<()> {
        payload.update_request_info(method, uri, headers, body);
        payload.update_language_info();

        self.update_payload_server_info(payload)?;

        Ok(())
    }

    #[cfg(feature = "wasm")]
    fn update_payload_server_info(&self, payload: &mut Payload) -> Result<()> {
        let protocol = host_get_protocol_version().map_err(|e| {
            log(
                LogLevel::Error,
                &format!("Failed to get protocol version: {}", e),
            );
            TreblleError::HostFunction(e.to_string())
        })?;

        payload.update_server_info(protocol);

        Ok(())
    }

    fn create_error_info(&self, status_code: u32) -> ErrorInfo {
        ErrorInfo {
            source: "response".to_string(),
            error_type: "HTTP Error".to_string(),
            message: format!("HTTP status code: {}", status_code),
            file: String::new(),
            line: 0,
        }
    }

    #[cfg(feature = "wasm")]
    fn send_to_treblle(&self, payload: &Payload, start_time: Instant) -> Result<()> {
        log(LogLevel::Debug, "Preparing to send data to Treblle API");

        let payload_json = payload.to_json()?;
        log(
            LogLevel::Debug,
            &format!("Payload JSON length: {}", payload_json.len()),
        );

        let mut http_client = HTTP_CLIENT.lock().map_err(|e| {
            log(
                LogLevel::Error,
                &format!("Failed to acquire HTTP_CLIENT lock: {}", e),
            );
            TreblleError::LockError(e.to_string())
        })?;

        http_client.post(payload_json.as_bytes(), &CONFIG.api_key)
            .map_err(|e| {
                log(
                    LogLevel::Error,
                    &format!("Failed to send data to Treblle API: {}", e),
                );
                TreblleError::Http(format!("Failed to send data to Treblle API: {}", e))
            })?;

        log(
            LogLevel::Debug,
            &format!(
                "Data sent successfully to Treblle API in {} ms",
                start_time.elapsed().as_millis()
            ),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::route_blacklist::RouteBlacklist;

    #[test]
    fn test_create_error_info() {
        let handler = HttpHandler;
        let error_info = handler.create_error_info(404);

        assert_eq!(error_info.source, "response");
        assert_eq!(error_info.error_type, "HTTP Error");
        assert_eq!(error_info.message, "HTTP status code: 404");
        assert!(error_info.file.is_empty());
        assert_eq!(error_info.line, 0);
    }
}
