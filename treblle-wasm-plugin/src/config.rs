//! Configuration module for the Treblle middleware.
//!
//! This module handles parsing and management of the middleware configuration.

use serde::Deserialize;
use serde_json::Value;

use crate::logger::{log, LogLevel};
use crate::constants::{DEFAULT_SENSITIVE_KEYS_REGEX, DEFAULT_TREBLLE_API_URLS};
use crate::error::{Result, TreblleError};

#[cfg(feature = "wasm")]
use crate::host_functions::host_get_config;

/// Represents the configuration for the Treblle middleware.
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub treblle_api_urls: Vec<String>,
    pub api_key: String,
    pub project_id: String,
    pub route_blacklist: Vec<String>,
    pub sensitive_keys_regex: String,
    pub buffer_response: bool,
    pub log_level: LogLevel,
    pub root_ca_path: Option<String>,
}

impl Config {
    /// Attempts to get the configuration, falling back to default values if unsuccessful.
    #[cfg(feature = "wasm")]
    pub fn get_or_fallback() -> Self {
        match Self::get() {
            Ok(config) => {
                if let Err(e) = config.validate() {
                    log(LogLevel::Error, &format!("Invalid configuration: {}", e));
                    Self::fallback()
                } else {
                    config
                }
            }
            Err(e) => {
                log(
                    LogLevel::Error,
                    &format!("Failed to parse config: {}, using fallback", e),
                );

                let fallback = Self::fallback();

                if let Err(e) = fallback.validate() {
                    log(
                        LogLevel::Error,
                        &format!("Fallback configuration is invalid: {}", e),
                    );
                }

                fallback
            }
        }
    }

    /// Retrieves the configuration from the host environment.
    #[cfg(feature = "wasm")]
    fn get() -> Result<Self> {
        let raw_config = host_get_config()?;
        let value: Value = serde_json::from_str(&raw_config).map_err(|e| TreblleError::Json(e))?;

        log(
            LogLevel::Debug,
            &format!("Received config from host: {}", value),
        );

        Ok(Self::from_value(value))
    }

    #[cfg(not(feature = "wasm"))]
    pub fn get_or_fallback() -> Self {
        Self::fallback()
    }

    #[cfg(not(feature = "wasm"))]
    fn get() -> Result<Self> {
        Ok(Self::fallback())
    }

    /// Constructs a Config instance from a serde_json::Value.
    fn from_value(value: Value) -> Self {
        Config {
            treblle_api_urls: value
                .get("treblleApiUrls")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(|| {
                    DEFAULT_TREBLLE_API_URLS
                        .iter()
                        .map(|&s| s.to_string())
                        .collect()
                }),

            api_key: value
                .get("apiKey")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),

            project_id: value
                .get("projectId")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),

            route_blacklist: value
                .get("routeBlacklist")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),

            sensitive_keys_regex: value
                .get("sensitiveKeysRegex")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| DEFAULT_SENSITIVE_KEYS_REGEX.to_string()),

            buffer_response: value
                .get("bufferResponse")
                .and_then(|v| {
                    v.as_bool()
                        .or_else(|| v.as_str().map(|s| s.to_lowercase() == "true"))
                })
                .unwrap_or(false),

            log_level: value
                .get("logLevel")
                .and_then(|v| v.as_str())
                .map(LogLevel::from_str)
                .unwrap_or_default(),

            root_ca_path: value
                .get("rootCaPath")
                .and_then(|v| v.as_str())
                .map(String::from),
        }
    }

    /// Returns a default configuration.
    fn fallback() -> Self {
        Config {
            treblle_api_urls: DEFAULT_TREBLLE_API_URLS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            api_key: String::new(),
            project_id: String::new(),
            route_blacklist: Vec::new(),
            sensitive_keys_regex: DEFAULT_SENSITIVE_KEYS_REGEX.to_string(),
            buffer_response: false,
            log_level: LogLevel::None,
            root_ca_path: None,
        }
    }

    /// Validates host-injected configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(TreblleError::Config("API key is required".to_string()));
        }

        if self.project_id.is_empty() {
            return Err(TreblleError::Config("Project ID is required".to_string()));
        }

        Ok(())
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
            "bufferResponse": true,
            "logLevel": "warn",
            "rootCaPath": "/etc/certs/rootCA.pem"
        });

        let config = Config::from_value(value);

        assert_eq!(
            config.treblle_api_urls,
            vec!["https://api1.treblle.com", "https://api2.treblle.com"]
        );
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.project_id, "test_project_id");
        assert_eq!(config.route_blacklist, vec!["/health", "/metrics"]);
        assert_eq!(config.sensitive_keys_regex, "password|secret");
        assert!(config.buffer_response);
        assert!(matches!(config.log_level, LogLevel::Warn));
        assert_eq!(config.root_ca_path, Some("/etc/certs/rootCA.pem".to_string()));
    }

    #[test]
    fn test_config_fallback() {
        let config = Config::fallback();

        assert_eq!(
            config.treblle_api_urls,
            DEFAULT_TREBLLE_API_URLS
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>()
        );
        assert!(config.api_key.is_empty());
        assert!(config.project_id.is_empty());
        assert!(config.route_blacklist.is_empty());
        assert_eq!(config.sensitive_keys_regex, DEFAULT_SENSITIVE_KEYS_REGEX);
        assert!(!config.buffer_response);
        assert!(matches!(config.log_level, LogLevel::None));
        assert_eq!(config.root_ca_path, None);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = Config {
            treblle_api_urls: vec![],
            api_key: "valid_key".to_string(),
            project_id: "valid_id".to_string(),
            route_blacklist: vec![],
            sensitive_keys_regex: "".to_string(),
            buffer_response: false,
            log_level: Default::default(),
            root_ca_path: None,
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = Config {
            treblle_api_urls: vec![],
            api_key: "".to_string(),
            project_id: "".to_string(),
            route_blacklist: vec![],
            sensitive_keys_regex: "".to_string(),
            buffer_response: false,
            log_level: Default::default(),
            root_ca_path: None,
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_log_level_parsing() {
        let test_cases = vec![
            ("debug", LogLevel::Debug),
            ("info", LogLevel::Info),
            ("warn", LogLevel::Warn),
            ("error", LogLevel::Error),
            ("none", LogLevel::None),
            ("invalid", LogLevel::None),
        ];

        for (input, _expected) in test_cases {
            let value = json!({ "logLevel": input });
            let config = Config::from_value(value);

            assert!(
                matches!(config.log_level, _expected),
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_log_level_case_insensitivity() {
        let value = json!({ "logLevel": "WARNING" });
        let config = Config::from_value(value);

        assert!(matches!(config.log_level, LogLevel::Warn));
    }

    #[test]
    fn test_log_level_default() {
        let value = json!({});
        let config = Config::from_value(value);

        assert!(matches!(config.log_level, LogLevel::None));
    }
}
