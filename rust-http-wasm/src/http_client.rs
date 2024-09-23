use crate::constants::{HTTP_TIMEOUT_SECONDS, LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::error::{Result, TreblleError};
use crate::host_functions::host_log;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use wasmedge_http_req::{request, uri::Uri};

pub struct HttpClient {
    urls: Vec<String>,
    current_index: AtomicUsize,
}

impl HttpClient {
    pub fn new(urls: Vec<String>) -> Self {
        HttpClient {
            urls,
            current_index: AtomicUsize::new(0),
        }
    }

    fn get_next_url(&self) -> &str {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst) % self.urls.len();
        &self.urls[index]
    }

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