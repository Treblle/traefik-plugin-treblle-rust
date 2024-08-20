use crate::constants::{LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::host_functions::host_log;
use std::{convert::TryFrom, time::Duration};
use wasmedge_http_req::{request, uri::Uri};

pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        HttpClient
    }

    pub fn send_to_treblle(&self, url: &str, payload: &[u8], api_key: &str) -> Result<(), String> {
        let mut writer = Vec::new();

        let uri: Uri = Uri::try_from(url).unwrap();

        let mut request = request::Request::new(&uri);

        request.method(request::Method::POST);
        request.header("Content-Type", "application/json");
        request.header("Content-Length", &payload.len().to_string());
        request.header("x-api-key", api_key);

        host_log(
            LOG_LEVEL_INFO,
            &format!("Sending request to Treblle API: {}", url),
        );
        host_log(
            LOG_LEVEL_INFO,
            &format!("Payload: {}", String::from_utf8_lossy(payload)),
        );

        let timeout = Some(Duration::from_secs(5));

        let response = request
            .body(payload)
            .timeout(timeout)
            .send(&mut writer)
            .map_err(|e| {
                let error_msg = format!("Failed to send POST request: {}", e);
                host_log(LOG_LEVEL_ERROR, &error_msg);
                error_msg
            })?;

        host_log(
            LOG_LEVEL_INFO,
            &format!("Response status: {}", response.status_code()),
        );
        host_log(
            LOG_LEVEL_INFO,
            &format!("Response body: {}", String::from_utf8_lossy(&writer)),
        );

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_msg = format!(
                "HTTP error: {}. Response body: {}",
                response.status_code(),
                String::from_utf8_lossy(&writer)
            );
            host_log(LOG_LEVEL_ERROR, &error_msg);
            Err(error_msg)
        }
    }
}
