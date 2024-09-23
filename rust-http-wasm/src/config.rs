use crate::constants::{DEFAULT_SENSITIVE_KEYS_REGEX, DEFAULT_TREBLLE_API_URLS, LOG_LEVEL_ERROR, LOG_LEVEL_INFO};
use crate::error::{Result, TreblleError};
use crate::host_functions::{host_get_config, host_log};
use serde::Deserialize;
use serde_json::Value;

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
    pub fn get_or_fallback() -> Self {
        Self::get().unwrap_or_else(|e| {
            host_log(
                LOG_LEVEL_ERROR,
                &format!("Failed to parse config: {}, using fallback", e),
            );
            Self::fallback()
        })
    }

    fn get() -> Result<Self> {
        let raw_config =
            host_get_config().map_err(|e| TreblleError::HostFunction(e.to_string()))?;
        let value: Value = serde_json::from_str(&raw_config)?;

        host_log(
            LOG_LEVEL_INFO,
            &format!("Received config from host: {}", value),
        );

        Ok(Self::from_value(value))
    }

    fn from_value(value: Value) -> Self {
        Config {
            treblle_api_urls: value
                .get("treblleApiUrls")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
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
                .unwrap_or("")
                .to_string(),

            project_id: value
                .get("projectId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),

            route_blacklist: value
                .get("routeBlacklist")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),

            sensitive_keys_regex: value
                .get("sensitiveKeysRegex")
                .and_then(|v| v.as_str())
                .unwrap_or(DEFAULT_SENSITIVE_KEYS_REGEX)
                .to_string(),

            buffer_response: value
                .get("bufferResponse")
                .and_then(|v| {
                    if let Some(b) = v.as_bool() {
                        Some(b)
                    } else if let Some(s) = v.as_str() {
                        Some(s.to_lowercase() == "true")
                    } else {
                        None
                    }
                })
                .unwrap_or(false),
        }
    }

    fn fallback() -> Self {
        Config {
            treblle_api_urls: DEFAULT_TREBLLE_API_URLS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            api_key: "".to_string(),
            project_id: "".to_string(),
            route_blacklist: vec![],
            sensitive_keys_regex: DEFAULT_SENSITIVE_KEYS_REGEX.to_string(),
            buffer_response: false,
        }
    }
}
