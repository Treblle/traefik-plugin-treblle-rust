//! HTTP client module for the Treblle middleware.
//!
//! This module handles sending data to the Treblle API.

use crate::constants::{HTTP_TIMEOUT_SECONDS, LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::error::{Result, TreblleError};
use crate::host_functions::host_log;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use wasmedge_http_req::{request, uri::Uri};

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
    pub fn post(&self, payload: &[u8], api_key: &str) -> Result<()> {
        let url = self.get_next_url();
        let timeout = Duration::from_secs(HTTP_TIMEOUT_SECONDS);
        let start_time = Instant::now();

        while start_time.elapsed() < timeout {
            match self.attempt_post(url, payload, api_key, timeout) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    host_log(
                        LOG_LEVEL_ERROR,
                        &format!("POST attempt failed: {}. Retrying...", e),
                    );
                    // Add a small delay before retrying
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }

        Err(TreblleError::Http("POST request timed out after all attempts".to_string()))
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
    fn attempt_post(&self, url: &str, payload: &[u8], api_key: &str, timeout: Duration) -> Result<()> {
        let mut writer = Vec::new();

        let uri = Uri::try_from(url)
            .map_err(|e| TreblleError::Http(format!("Invalid URL: {}", e)))?;
        let mut request = request::Request::new(&uri);

        request.method(request::Method::POST);
        request.header("Content-Type", "application/json");
        request.header("X-Api-Key", api_key);
        request.header("Content-Length", &payload.len().to_string());

        host_log(LOG_LEVEL_INFO, &format!("Sending payload to URL: {}", url));

        let response = request
            .body(payload)
            .timeout(Some(timeout))
            .send(&mut writer)
            .map_err(|e| TreblleError::Http(format!("Failed to send POST request: {}", e)))?;

        host_log(
            LOG_LEVEL_INFO,
            &format!(
                "Received response from Treblle API: status {}",
                response.status_code()
            ),
        );

        if response.status_code().is_success() {
            host_log(LOG_LEVEL_INFO, "Successfully sent data");
            Ok(())
        } else {
            let response_body = String::from_utf8_lossy(&writer);
            let error_msg = format!(
                "HTTP error: {}. Response body: {}",
                response.status_code(),
                response_body
            );

            host_log(LOG_LEVEL_ERROR, &error_msg);
            Err(TreblleError::Http(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        HttpRequest {
            fn method(&mut self, _method: request::Method) -> &mut Self;
            fn header(&mut self, _name: &str, _value: &str) -> &mut Self;
            fn body(&mut self, _body: &[u8]) -> &mut Self;
            fn timeout(&mut self, _timeout: Option<Duration>) -> &mut Self;
            fn send(&self, _writer: &mut Vec<u8>) -> std::result::Result<request::Response, wasmedge_http_req::Error>;
        }
    }

    #[test]
    fn test_get_next_url() {
        let client = HttpClient::new(vec!["url1".to_string(), "url2".to_string()]);
        assert_eq!(client.get_next_url(), "url1");
        assert_eq!(client.get_next_url(), "url2");
        assert_eq!(client.get_next_url(), "url1");
    }

    #[test]
    fn test_post_success() {
        let mut mock_request = MockHttpRequest::new();
        mock_request.expect_method()
            .with(eq(request::Method::POST))
            .return_const(&mut MockHttpRequest::new());
        mock_request.expect_header()
            .times(3)
            .return_const(&mut MockHttpRequest::new());
        mock_request.expect_body()
            .return_const(&mut MockHttpRequest::new());
        mock_request.expect_timeout()
            .return_const(&mut MockHttpRequest::new());
        mock_request.expect_send()
            .return_once(|_| Ok(request::Response::new(200, vec![], vec![])));

        // You might need to adjust how you inject this mock into your HttpClient
        // This is a simplified example
        let client = HttpClient::new(vec!["http://api.treblle.com".to_string()]);
        let result = client.post(b"test payload", "test_api_key");
        assert!(result.is_ok());
    }
}