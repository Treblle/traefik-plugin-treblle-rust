//! Error types for the Treblle middleware.
//!
//! This module defines the custom error types used throughout the application.

use thiserror::Error;

/// Represents errors that can occur in the Treblle middleware.
#[derive(Error, Debug)]
pub enum TreblleError {
    /// Represents HTTP-related errors.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Represents errors that occur during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Represents I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Represents errors related to regular expressions.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Represents errors that occur when interacting with host functions.
    #[error("Host function error: {0}")]
    HostFunction(String),

    /// Represents configuration-related errors.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Represents errors related to payload processing.
    #[error("Payload error: {0}")]
    Payload(String),

    /// Represents unexpected or unhandled errors.
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// A specialized Result type for Treblle operations.
pub type Result<T> = std::result::Result<T, TreblleError>;

impl TreblleError {
    /// Returns a string representation of the error type.
    pub fn error_type(&self) -> String {
        match self {
            TreblleError::Http(_) => "HTTP Error".to_string(),
            TreblleError::Json(_) => "JSON Error".to_string(),
            TreblleError::Io(_) => "I/O Error".to_string(),
            TreblleError::Regex(_) => "Regex Error".to_string(),
            TreblleError::HostFunction(_) => "Host Function Error".to_string(),
            TreblleError::Config(_) => "Configuration Error".to_string(),
            TreblleError::Payload(_) => "Payload Error".to_string(),
            TreblleError::Unexpected(_) => "Unexpected Error".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::de::Error;
    use super::*;

    #[test]
    fn test_error_type() {
        assert_eq!(TreblleError::Http("test".to_string()).error_type(), "HTTP Error");
        assert_eq!(TreblleError::Json(serde_json::Error::custom("test")).error_type(), "JSON Error");
        assert_eq!(TreblleError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")).error_type(), "I/O Error");
        assert_eq!(TreblleError::Regex(regex::Error::Syntax("test".to_string())).error_type(), "Regex Error");
        assert_eq!(TreblleError::HostFunction("test".to_string()).error_type(), "Host Function Error");
        assert_eq!(TreblleError::Config("test".to_string()).error_type(), "Configuration Error");
        assert_eq!(TreblleError::Payload("test".to_string()).error_type(), "Payload Error");
        assert_eq!(TreblleError::Unexpected("test".to_string()).error_type(), "Unexpected Error");
    }

    #[test]
    fn test_error_display() {
        let error = TreblleError::Http("Not Found".to_string());
        assert_eq!(format!("{}", error), "HTTP error: Not Found");
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "IO Error");
        let treblle_error: TreblleError = io_error.into();
        assert!(matches!(treblle_error, TreblleError::Io(_)));

        let json_error = serde_json::Error::custom("JSON Error");
        let treblle_error: TreblleError = json_error.into();
        assert!(matches!(treblle_error, TreblleError::Json(_)));
    }
}