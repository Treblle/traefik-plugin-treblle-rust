//! Utility functions for parsing and masking data.

use chrono::Utc;
use regex::Regex;
use serde_json::{Map, Value};
use std::time::Instant;

use crate::config::Config;
use crate::error::{Result, TreblleError};
use crate::schema::{RequestInfo, ResponseInfo};

/// Parses and processes request information.
///
/// # Arguments
///
/// * `method` - The HTTP method of the request.
/// * `uri` - The URI of the request.
/// * `headers` - A map of request headers.
/// * `body` - The raw body of the request.
/// * `config` - The configuration containing sensitive data patterns.
///
/// # Returns
///
/// Returns a `Result<RequestInfo>` containing the processed request information,
/// or an error if processing fails.
pub fn parse_request(
    method: String,
    uri: String,
    headers: std::collections::HashMap<String, String>,
    body: &[u8],
    config: &Config,
) -> Result<RequestInfo> {
    let ip = extract_ip_from_headers(&headers).unwrap_or_else(|| "Unknown".to_string());
    let user_agent = headers.get("User-Agent").cloned().unwrap_or_default();

    let parsed_body = parse_json_body(body);
    let masked_body = mask_sensitive_data(&parsed_body, &config.sensitive_keys_regex)?;
    let masked_headers = mask_sensitive_headers(&headers, &config.sensitive_keys_regex)?;

    Ok(RequestInfo {
        timestamp: Utc::now().to_rfc3339(),
        ip,
        url: uri,
        user_agent,
        method,
        headers: masked_headers,
        body: serde_json::to_value(masked_body).ok(),
    })
}

/// Parses and processes response information.
///
/// # Arguments
///
/// * `status` - The HTTP status code of the response.
/// * `headers` - A map of response headers.
/// * `body` - The raw body of the response.
/// * `start_time` - The time when the request processing started.
/// * `config` - The configuration containing sensitive data patterns.
///
/// # Returns
///
/// Returns a `Result<ResponseInfo>` containing the processed response information,
/// or an error if processing fails.
pub fn parse_response(
    status: u32,
    headers: std::collections::HashMap<String, String>,
    body: &[u8],
    start_time: Instant,
    config: &Config,
) -> Result<ResponseInfo> {
    let parsed_body = parse_json_body(body);
    let masked_body = mask_sensitive_data(&parsed_body, &config.sensitive_keys_regex)?;
    let masked_headers = mask_sensitive_headers(&headers, &config.sensitive_keys_regex)?;

    Ok(ResponseInfo {
        headers: masked_headers,
        code: status,
        size: body.len(),
        load_time: start_time.elapsed().as_secs_f64(),
        body: serde_json::to_value(masked_body).ok(),
    })
}

/// Parses a JSON body, returning a null value if parsing fails.
///
/// # Arguments
///
/// * `body` - The raw body to parse as JSON.
///
/// # Returns
///
/// Returns a `Value` representing the parsed JSON, or `Value::Null` if parsing fails.
pub fn parse_json_body(body: &[u8]) -> Value {
    serde_json::from_slice(body).unwrap_or(Value::Null)
}

/// Masks sensitive data in a JSON value based on a regex pattern.
///
/// # Arguments
///
/// * `data` - The JSON value to mask.
/// * `sensitive_keys_regex` - The regex pattern to identify sensitive keys.
///
/// # Returns
///
/// Returns a `Result<Value>` containing the masked JSON value,
/// or an error if the regex pattern is invalid.
pub fn mask_sensitive_data(data: &Value, sensitive_keys_regex: &str) -> Result<Value> {
    let re = Regex::new(sensitive_keys_regex)
        .map_err(|e| TreblleError::Regex(e))?;

    Ok(match data {
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (key, value) in map {
                if re.is_match(key) {
                    new_map.insert(key.clone(), Value::String("*****".to_string()));
                } else {
                    new_map.insert(
                        key.clone(),
                        mask_sensitive_data(value, sensitive_keys_regex)?,
                    );
                }
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(
            arr.iter()
                .map(|v| mask_sensitive_data(v, sensitive_keys_regex))
                .collect::<Result<Vec<_>>>()?,
        ),
        _ => data.clone(),
    })
}

/// Masks sensitive headers based on a regex pattern.
///
/// # Arguments
///
/// * `headers` - A map of headers to mask.
/// * `sensitive_keys_regex` - The regex pattern to identify sensitive header names.
///
/// # Returns
///
/// Returns a `Result<HashMap<String, String>>` containing the masked headers,
/// or an error if the regex pattern is invalid.
pub fn mask_sensitive_headers(
    headers: &std::collections::HashMap<String, String>,
    sensitive_keys_regex: &str,
) -> Result<std::collections::HashMap<String, String>> {
    let re = Regex::new(sensitive_keys_regex)
        .map_err(|e| TreblleError::Regex(e))?;

    Ok(headers
        .iter()
        .map(|(key, value)| {
            if re.is_match(key) {
                (key.clone(), "*****".to_string())
            } else {
                (key.clone(), value.clone())
            }
        })
        .collect())
}

/// Extracts IP address from headers.
///
/// # Arguments
///
/// * `headers` - A map of headers to search for IP information.
///
/// # Returns
///
/// Returns an `Option<String>` containing the extracted IP address,
/// or `None` if no IP address is found.
pub fn extract_ip_from_headers(headers: &std::collections::HashMap<String, String>) -> Option<String> {
    headers
        .get("X-Forwarded-For")
        .or_else(|| headers.get("X-Real-IP"))
        .map(|ip| ip.split(',').next().unwrap_or("").trim().to_string())
}

/// Checks if a content type is JSON.
///
/// # Arguments
///
/// * `content_type` - The content type string to check.
///
/// # Returns
///
/// Returns `true` if the content type indicates JSON, `false` otherwise.

pub fn is_json(content_type: &str) -> bool {
    content_type.to_lowercase().contains("application/json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_json() {
        assert!(is_json("application/json"));
        assert!(is_json("Application/JSON"));
        assert!(!is_json("text/plain"));
    }

    #[test]
    fn test_mask_sensitive_data() -> Result<()> {
        let data = serde_json::json!({
            "username": "john_doe",
            "password": "secret123",
            "email": "john@example.com"
        });
        let masked = mask_sensitive_data(&data, r"password|email")?;
        assert_eq!(masked["username"], "john_doe");
        assert_eq!(masked["password"], "*****");
        assert_eq!(masked["email"], "*****");
        Ok(())
    }

    #[test]
    fn test_mask_sensitive_headers() -> Result<()> {
        let mut headers = std::collections::HashMap::new();
        headers.insert("User-Agent".to_string(), "TestAgent".to_string());
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let masked = mask_sensitive_headers(&headers, r"Authorization")?;

        assert_eq!(masked["User-Agent"], "TestAgent");
        assert_eq!(masked["Authorization"], "*****");
        Ok(())
    }

    #[test]
    fn test_invalid_regex() {
        let data = serde_json::json!({"key": "value"});
        let result = mask_sensitive_data(&data, r"[invalid regex");
        assert!(result.is_err());
    }
}