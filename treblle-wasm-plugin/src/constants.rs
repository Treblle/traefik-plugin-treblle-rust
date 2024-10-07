#![allow(dead_code, unused_variables)]

pub const LOG_LEVEL_DEBUG: i32 = -1;
pub const LOG_LEVEL_INFO: i32 = 0;
pub const LOG_LEVEL_WARN: i32 = 1;
pub const LOG_LEVEL_ERROR: i32 = 2;
pub const LOG_LEVEL_NONE: i32 = 3;

pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const REQUEST_KIND: u32 = 0;
pub const RESPONSE_KIND: u32 = 1;

pub const DEFAULT_TREBLLE_API_URLS: [&str; 3] = [
    "https://rocknrolla.treblle.com",
    "https://punisher.treblle.com",
    "https://sicario.treblle.com",
];

pub const DEFAULT_SENSITIVE_KEYS_REGEX: &str =
    r"(?i)(password|pwd|secret|password_confirmation|cc|card_number|ccv|ssn|credit_score)";

pub const HTTP_TIMEOUT_SECONDS: u64 = 10;
