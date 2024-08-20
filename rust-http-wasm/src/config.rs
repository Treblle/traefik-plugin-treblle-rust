use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::constants::LOG_LEVEL_ERROR;
use crate::host_functions::{host_get_config, host_log};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub treblle_api_url: String,
    pub api_key: String,
    pub project_id: String,
    pub route_blacklist: Vec<String>,
    pub sensitive_keys_regex: String,
    pub request_url: String,
    pub allowed_content_type: String,
}

impl Config {
    pub fn get_or_fallback() -> Self {
        match Self::get() {
            Ok(config) => config,
            Err(e) => {
                host_log(
                    LOG_LEVEL_ERROR,
                    &format!("Failed to parse config: {}, using fallback", e),
                );
                Self::fallback()
            }
        }
    }

    fn get() -> Result<Self> {
        let raw_config = host_get_config().context("Failed to get config from host")?;
        let value: Value =
            serde_json::from_str(&raw_config).context("Failed to parse config JSON")?;
        Ok(Self::from_value(value))
    }

    fn from_value(value: Value) -> Self {
        Config {
            treblle_api_url: value.get("treblleApiUrl").and_then(|v| v.as_str()).unwrap_or("http://treblle-api:3002/api").to_string(),
            api_key: value.get("apiKey").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            project_id: value.get("projectId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            route_blacklist: value.get("routeBlacklist").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
            sensitive_keys_regex: value.get("sensitiveKeysRegex").and_then(|v| v.as_str()).unwrap_or(r"(?i)(password|pwd|secret|password_confirmation|cc|card_number|ccv|ssn|credit_score)").to_string(),
            request_url: value.get("requestUrl").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            allowed_content_type: value.get("allowedContentType").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        }
    }

    fn fallback() -> Self {
        Config {
            treblle_api_url: "http://treblle-api:3002/api".to_string(),
            api_key: "".to_string(),
            project_id: "".to_string(),
            route_blacklist: vec![],
            sensitive_keys_regex: r"(?i)(password|pwd|secret|password_confirmation|cc|card_number|ccv|ssn|credit_score)".to_string(),
            request_url: "".to_string(),
            allowed_content_type: "application/json".to_string(),
        }
    }
}
