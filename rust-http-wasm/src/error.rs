//! Error types for the Treblle middleware.
use thiserror::Error;

#[cfg(not(feature = "wasm"))]
use reqwest::Error;

/// Represents errors that can occur in the Treblle middleware.
#[derive(Error, Debug)]
pub enum TreblleError {
    /// Represents HTTP-related errors.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Represents errors that occur during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Represents errors related to regular expressions.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Represents errors that occur when interacting with host functions.
    #[error("Host function error: {0}")]
    HostFunction(String),

    /// Represents configuration-related errors.
    #[error("Configuration error: {0}")]
    Config(String),

    #[cfg(not(feature = "wasm"))]
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] Error),
}

/// A specialized Result type for Treblle operations.
pub type Result<T> = std::result::Result<T, TreblleError>;

#[cfg(test)]
mod tests {
    use serde::de::Error;
    use super::*;

    #[test]
    fn test_error_display() {
        let error = TreblleError::Http("Not Found".to_string());
        assert_eq!(format!("{}", error), "HTTP error: Not Found");
    }

    #[test]
    fn test_error_conversion() {
        let json_error = serde_json::Error::custom("JSON Error");
        let treblle_error: TreblleError = json_error.into();
        assert!(matches!(treblle_error, TreblleError::Json(_)));
    }
}