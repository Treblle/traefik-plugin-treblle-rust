//! Payload module for the Treblle middleware.
//!
//! This module handles the creation and manipulation of the payload
//! that will be sent to the Treblle API.

use std::collections::HashMap;
use std::time::Instant;

use crate::error::Result;
use crate::schema::*;
use crate::utils;
use crate::CONFIG;

/// Represents the payload that will be sent to the Treblle API.
pub struct Payload {
    pub data: TrebllePayload,
}

impl Payload {
    /// Creates a new Payload instance.
    pub fn new() -> Self {
        Payload {
            data: TrebllePayload {
                api_key: CONFIG.api_key.clone(),
                project_id: CONFIG.project_id.clone(),
                version: 0.6,
                sdk: "rust-wasm".to_string(),
                data: PayloadData::default(),
            },
        }
    }

    /// Updates the request information in the payload.
    pub fn update_request_info(
        &mut self,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: &[u8],
    ) {
        self.data.data.request = utils::parse_request(method, url, headers, body, &CONFIG)
            .expect("Error parsing request");
    }

    /// Updates the response information in the payload.
    pub fn update_response_info(
        &mut self,
        status: u32,
        headers: HashMap<String, String>,
        body: &[u8],
        start_time: Instant,
    ) {
        self.data.data.response = utils::parse_response(status, headers, body, start_time, &CONFIG)
            .expect("Error parsing response");
    }

    /// Adds an error to the payload.
    pub fn add_error(&mut self, error: ErrorInfo) {
        self.data.data.errors.push(error);
    }

    /// Converts the payload to a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.data).map_err(Into::into)
    }

    /// Updates the server information in the payload.
    /// Although this function exists here for proper & complete schema validated response,
    /// it's not going to see much use within WASM environment. 
    /// 
    /// This part of payload schema is intended for SDK/middleware use.
    pub fn update_server_info(&mut self, protocol: String) {
        self.data.data.server = ServerInfo {
            ip: "Unknown".to_string(), // TODO: Implement server IP detection
            timezone: chrono::Local::now().format("%Z").to_string(),
            software: None,
            signature: None,
            protocol,
            encoding: None,
            os: OsInfo {
                name: std::env::consts::OS.to_string(),
                release: "Unknown".to_string(), // TODO: Implement OS release detection
                architecture: std::env::consts::ARCH.to_string(),
            },
        };
    }

    /// Updates the language information in the payload.
    pub fn update_language_info(&mut self) {
        self.data.data.language = LanguageInfo {
            name: "rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            expose_php: None,
            display_errors: None,
        };
    }
}

/// Checks if the given content type is JSON.
pub fn is_json(content_type: &str) -> bool {
    utils::is_json(content_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::constants::DEFAULT_SENSITIVE_KEYS_REGEX;
    use crate::logger::LogLevel;

    fn create_test_config() -> Config {
        Config {
            treblle_api_urls: vec!["https://api.treblle.com".to_string()],
            api_key: "test_api_key".to_string(),
            project_id: "test_project_id".to_string(),
            route_blacklist: vec![],
            sensitive_keys_regex: DEFAULT_SENSITIVE_KEYS_REGEX.to_string(),
            buffer_response: false,
            log_level: LogLevel::None,
        }
    }

    #[test]
    fn test_payload_new() {
        let config = create_test_config();

        let payload = Payload {
            data: TrebllePayload {
                api_key: config.api_key.clone(),
                project_id: config.project_id.clone(),
                version: 0.6,
                sdk: "rust-wasm".to_string(),
                data: PayloadData::default(),
            },
        };

        assert_eq!(payload.data.api_key, config.api_key);
        assert_eq!(payload.data.project_id, config.project_id);
        assert_eq!(payload.data.version, 0.6);
        assert_eq!(payload.data.sdk, "rust-wasm");
    }

    #[test]
    fn test_update_request_info() {
        let config = create_test_config();

        let mut payload = Payload {
            data: TrebllePayload {
                api_key: config.api_key.clone(),
                project_id: config.project_id.clone(),
                version: 0.6,
                sdk: "rust-wasm".to_string(),
                data: PayloadData::default(),
            },
        };

        let method = "GET".to_string();
        let url = "https://api.example.com/test".to_string();
        let headers = HashMap::new();
        let body = b"test body";

        payload.update_request_info(method.clone(), url.clone(), headers, body);

        assert_eq!(payload.data.data.request.method, method);
        assert_eq!(payload.data.data.request.url, url);
    }

    #[test]
    fn test_is_json() {
        assert!(is_json("application/json"));
        assert!(is_json("application/json; charset=utf-8"));
        assert!(!is_json("text/plain"));
    }
}
