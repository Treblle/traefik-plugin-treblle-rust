//! HTTP client module for the Treblle middleware.
//!
//! This module handles sending data to the Treblle API.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[cfg(feature = "wasm")]
use wasmedge_http_req::{request, uri::Uri};

use crate::constants::HTTP_TIMEOUT_SECONDS;
use crate::error::{Result, TreblleError};
use crate::logger::{log, LogLevel};

/// Represents an HTTP client for sending data to Treblle API.
pub struct HttpClient {
    urls: Vec<String>,
    current_index: AtomicUsize,
}

impl HttpClient {
    /// Creates a new HttpClient instance.
    ///
    /// # Arguments
    ///
    /// * `urls` - A vector of Treblle API URLs to use for sending data.
    pub fn new(urls: Vec<String>) -> Self {
        HttpClient {
            urls,
            current_index: AtomicUsize::new(0),
        }
    }

    /// Gets the next URL to use in a round-robin fashion.
    fn get_next_url(&self) -> &str {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst) % self.urls.len();
        &self.urls[index]
    }

    /// Sends a POST request to the Treblle API.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload to send.
    /// * `api_key` - The API key to use for authentication.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was successful, or an error otherwise.
    #[cfg(feature = "wasm")]
    pub fn post(&self, payload: &[u8], api_key: &str) -> Result<()> {
        let url = self.get_next_url();
        let timeout = Duration::from_secs(HTTP_TIMEOUT_SECONDS);

        self.attempt_post(url, payload, api_key, timeout)?;
        
        Ok(())
    }
    
    /// Attempts to send a POST request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to send the request to.
    /// * `payload` - The payload to send.
    /// * `api_key` - The API key to use for authentication.
    /// * `timeout` - The timeout duration for the request.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was successful, or an error otherwise.
    #[cfg(feature = "wasm")]
    fn attempt_post(
        &self,
        url: &str,
        payload: &[u8],
        api_key: &str,
        timeout: Duration,
    ) -> Result<()> {
        let mut writer = Vec::new();

        let uri =
            Uri::try_from(url).map_err(|e| TreblleError::Http(format!("Invalid URL: {}", e)))?;
        
        let mut request = request::Request::new(&uri);

        request.method(request::Method::POST);
        request.header("Content-Type", "application/json");
        request.header("X-Api-Key", api_key);
        request.header("Content-Length", &payload.len().to_string());

        // Set the root certificate file path
        let cert_path = Path::new("/etc/certs/rootCA.pem");
        request.root_cert_file_pem(cert_path);

        log(LogLevel::Debug, &format!("Sending payload to URL: {}", url));

        let response = request
            .body(payload)
            .timeout(Some(timeout))
            .send(&mut writer)
            .map_err(|e| TreblleError::Http(format!("Failed to send POST request: {}", e)))?;

        log(
            LogLevel::Debug,
            &format!(
                "Received response from Treblle API: status {}",
                response.status_code()
            ),
        );

        log(LogLevel::Debug, "Response headers:");
        for (key, value) in response.headers().iter() {
            log(LogLevel::Debug, &format!("  {}: {}", key, value));
        }

        let response_body = String::from_utf8_lossy(&writer);
        log(LogLevel::Debug, "Response body:");
        match serde_json::from_str::<serde_json::Value>(&response_body) {
            Ok(json) => {
                match serde_json::to_string_pretty(&json) {
                    Ok(pretty) => log(LogLevel::Debug, &pretty),
                    Err(_) => log(LogLevel::Debug, &response_body),
                }
            }
            Err(_) => log(LogLevel::Debug, &response_body),
        }

        if response.status_code().is_success() {
            log(LogLevel::Debug, "Successfully sent data");
            Ok(())
        } else {
            let response_body = String::from_utf8_lossy(&writer);
            let error_msg = format!(
                "HTTP error: {}. Response body: {}",
                response.status_code(),
                response_body
            );

            log(LogLevel::Error, &error_msg);
            Err(TreblleError::Http(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_url() {
        let client = HttpClient::new(vec!["url1".to_string(), "url2".to_string()]);

        assert_eq!(client.get_next_url(), "url1");
        assert_eq!(client.get_next_url(), "url2");
        assert_eq!(client.get_next_url(), "url1");
    }

    #[test]
    fn test_http_client_creation() {
        let urls = vec![
            "https://api1.treblle.com".to_string(),
            "https://api2.treblle.com".to_string(),
        ];
        let client = HttpClient::new(urls.clone());

        assert_eq!(client.urls, urls);
        assert_eq!(client.current_index.load(Ordering::SeqCst), 0);
    }

    // Note: We can't easily test the `post` method without mocking external dependencies.
    // For thorough testing, we should be using integration tests or a different testing strategy
    // that doesn't rely on mocking HTTP requests in a WASM environment.
}
