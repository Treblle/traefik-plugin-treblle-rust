//! Configuration module for the Treblle middleware.
//!
//! This module handles parsing and management of the middleware configuration.

use crate::constants::{DEFAULT_SENSITIVE_KEYS_REGEX, DEFAULT_TREBLLE_API_URLS, LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::error::{Result, TreblleError};
use crate::host_functions::{host_get_config, host_log};
use serde::Deserialize;
use serde_json::Value;

/// Represents the configuration for the Treblle middleware.
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub treblle_api_urls: Vec<String>,
    pub api_key: String,
    pub project_id: String,
    pub route_blacklist: Vec<String>,
    pub sensitive_keys_regex: String,
    pub buffer_response: bool,
}

impl Config {
    /// Attempts to get the configuration, falling back to default values if unsuccessful.
    pub fn get_or_fallback() -> Self {
        Self::get().unwrap_or_else(|e| {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to parse config: {}, using fallback", e),
            );
            Self::fallback()
        })
    }

    /// Retrieves the configuration from the host environment.
    fn get() -> Result<Self> {
        let raw_config = host_get_config()?;
        let value: Value = serde_json::from_str(&raw_config)
            .map_err(|e| TreblleError::Json(e))?;

        host_log(
            LOG_LEVEL_INFO,
            &format!("Received config from host: {}", value),
        );

        Ok(Self::from_value(value))
    }

    /// Constructs a Config instance from a serde_json::Value.
    fn from_value(value: Value) -> Self {
        Config {
            treblle_api_urls: value.get("treblleApiUrls")
                .and_then(|v| v.as_array())
                .map(|a| a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect())
                .unwrap_or_else(|| DEFAULT_TREBLLE_API_URLS.iter().map(|&s| s.to_string()).collect()),

            api_key: value.get("apiKey")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),

            project_id: value.get("projectId")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),

            route_blacklist: value.get("routeBlacklist")
                .and_then(|v| v.as_array())
                .map(|a| a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect())
                .unwrap_or_default(),

            sensitive_keys_regex: value.get("sensitiveKeysRegex")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| DEFAULT_SENSITIVE_KEYS_REGEX.to_string()),

            buffer_response: value.get("bufferResponse")
                .and_then(|v| {
                    v.as_bool().or_else(|| {
                        v.as_str().map(|s| s.to_lowercase() == "true")
                    })
                })
                .unwrap_or(false),
        }
    }

    /// Returns a default configuration.
    fn fallback() -> Self {
        Config {
            treblle_api_urls: DEFAULT_TREBLLE_API_URLS.iter().map(|&s| s.to_string()).collect(),
            api_key: String::new(),
            project_id: String::new(),
            route_blacklist: Vec::new(),
            sensitive_keys_regex: DEFAULT_SENSITIVE_KEYS_REGEX.to_string(),
            buffer_response: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_from_value() {
        let value = json!({
            "treblleApiUrls": ["https://api1.treblle.com", "https://api2.treblle.com"],
            "apiKey": "test_api_key",
            "projectId": "test_project_id",
            "routeBlacklist": ["/health", "/metrics"],
            "sensitiveKeysRegex": "password|secret",
            "bufferResponse": true
        });

        let config = Config::from_value(value);

        assert_eq!(config.treblle_api_urls, vec!["https://api1.treblle.com", "https://api2.treblle.com"]);
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.project_id, "test_project_id");
        assert_eq!(config.route_blacklist, vec!["/health", "/metrics"]);
        assert_eq!(config.sensitive_keys_regex, "password|secret");
        assert!(config.buffer_response);
    }

    #[test]
    fn test_config_fallback() {
        let config = Config::fallback();

        assert_eq!(config.treblle_api_urls, DEFAULT_TREBLLE_API_URLS.iter().map(|&s| s.to_string()).collect::<Vec<String>>());
        assert!(config.api_key.is_empty());
        assert!(config.project_id.is_empty());
        assert!(config.route_blacklist.is_empty());
        assert_eq!(config.sensitive_keys_regex, DEFAULT_SENSITIVE_KEYS_REGEX);
        assert!(!config.buffer_response);
    }
}