#![allow(dead_code)]

pub const LOG_LEVEL_ERROR: i32 = 2;
pub const LOG_LEVEL_WARN: i32 = 1;
pub const LOG_LEVEL_INFO: i32 = 0;
pub const LOG_LEVEL_DEBUG: i32 = -1;

pub const DEFAULT_TREBLLE_API_URL: &str = "http://treblle-api:3002/api";
pub const DEFAULT_SENSITIVE_KEYS_REGEX: &str =
    r"(?i)(password|pwd|secret|password_confirmation|cc|card_number|ccv|ssn|credit_score)";
pub const HTTP_TIMEOUT_SECONDS: u64 = 10;
