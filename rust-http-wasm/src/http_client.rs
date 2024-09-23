use crate::constants::{HTTP_TIMEOUT_SECONDS, LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::error::{Result, TreblleError};
use crate::host_functions::host_log;
use std::time::Duration;
use wasmedge_http_req::{request, response::Response, uri::Uri};

pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        HttpClient
    }

    pub fn post(&self, url: &str, payload: &[u8], api_key: &str) -> Result<()> {
        let mut writer = Vec::new();
        let timeout = Duration::from_secs(HTTP_TIMEOUT_SECONDS);

        let uri =
            Uri::try_from(url).map_err(|e| TreblleError::Http(format!("Invalid URL: {}", e)))?;
        let mut request = request::Request::new(&uri);

        request.method(request::Method::POST);
        request.header("Content-Type", "application/json");
        request.header("x-api-key", api_key);
        request.header("Content-Length", &payload.len().to_string());

        host_log(
            LOG_LEVEL_INFO,
            &format!("Payload: {}", String::from_utf8_lossy(payload)),
        );

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

    pub fn get(&self, url: &str) -> Result<Response> {
        let timeout = Duration::from_secs(HTTP_TIMEOUT_SECONDS);

        host_log(
            LOG_LEVEL_INFO,
            &format!("Preparing GET request to: {}", url),
        );

        let uri = Uri::try_from(url).map_err(|e| {
            host_log(LOG_LEVEL_ERROR, &format!("Failed to parse URL: {}", e));
            TreblleError::Http(format!("Invalid URL: {}", e))
        })?;

        let mut request = request::Request::new(&uri);

        let mut writer = Vec::new();
        let response = request
            .timeout(Some(timeout))
            .send(&mut writer)
            .map_err(|e| {
                host_log(LOG_LEVEL_ERROR, &format!("Failed to send request: {:?}", e));
                TreblleError::Http(format!("Failed to send GET request: {:?}", e))
            })?;

        host_log(
            LOG_LEVEL_INFO,
            &format!("Received response: status {}", response.status_code()),
        );

        Ok(response)
    }
}
