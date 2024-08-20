use thiserror::Error;

#[derive(Error, Debug)]
pub enum TreblleError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Host function error: {0}")]
    HostFunction(String),
}

pub type Result<T> = std::result::Result<T, TreblleError>;
