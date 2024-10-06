//! Error types for the Treblle middleware.

use std::fmt;
use std::fmt::write;
use std::io;

use thiserror::Error;

/// Represents errors that can occur in the Treblle middleware.
#[derive(Error, Debug)]
pub enum TreblleError {
    /// Represents I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Represents HTTP-related errors.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Represents JSON parsing or serialization errors.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Represents errors related to invalid URLs.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Represents errors when a hostname is invalid for TLS.
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    /// Represents TLS-related errors.
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    /// Represents TCP-related errors.
    #[error("TCP error: {0}")]
    Tcp(String),

    /// Represents errors related to certificate handling.
    #[error("Certificate error: {0}")]
    Certificate(String),

    /// Represents timeout errors.
    #[error("Operation timed out")]
    Timeout,

    /// Represents configuration-related errors.
    #[error("Config error: {0}")]
    Config(String),

    /// Represents errors related to regular expressions.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Represents errors that occur when interacting with host functions.
    #[error("Host function error: {0}")]
    HostFunction(String),

}


/// A `Result` type alias for operations that can result in a `TreblleError`.
pub type Result<T> = std::result::Result<T, TreblleError>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Error;

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