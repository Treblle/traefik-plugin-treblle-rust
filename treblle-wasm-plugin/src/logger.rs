//! Logging module for the Treblle middleware.
//!
//! This module provides a unified logging interface that works in both WASM and non-WASM environments.
//! It uses the `log` crate for non-WASM environments and falls back to `host_log` for WASM environments.

use serde::Deserialize;

use std::sync::atomic::{AtomicI32, Ordering};

use crate::constants::{
    LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_NONE, LOG_LEVEL_WARN,
};

#[cfg(feature = "wasm")]
use crate::CONFIG;

#[cfg(feature = "wasm")]
use crate::host_functions::host_log;

#[cfg(not(feature = "wasm"))]
use log::{debug, error, info, warn};

static LOG_LEVEL: AtomicI32 = AtomicI32::new(LOG_LEVEL_INFO);

#[derive(Deserialize, Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    None,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::None
    }
}

impl LogLevel {
    pub fn as_i32(&self) -> i32 {
        match self {
            LogLevel::Debug => LOG_LEVEL_DEBUG,
            LogLevel::Info => LOG_LEVEL_INFO,
            LogLevel::Warn => LOG_LEVEL_WARN,
            LogLevel::Error => LOG_LEVEL_ERROR,
            LogLevel::None => LOG_LEVEL_NONE,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "none" => LogLevel::None,
            _ => LogLevel::default(),
        }
    }
}

#[cfg(not(feature = "wasm"))]
impl From<LogLevel> for log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
            LogLevel::None => log::Level::Error, // Map None to Error in non-WASM environments
        }
    }
}

/// Initialize the logger.
///
/// In non-WASM environments, this initializes env_logger.
/// In WASM environments, this is a no-op as logging is handled by the host.
#[cfg(not(feature = "wasm"))]
pub fn init() {
    env_logger::init();
}

#[cfg(feature = "wasm")]
pub fn init() {
    let level = CONFIG.log_level.as_i32();
    LOG_LEVEL.store(level, Ordering::Relaxed);
    log(LogLevel::Debug, &format!("Log level set to: {:?}", CONFIG.log_level));
}

/// Log a message at the specified level.
///
/// This function checks the configured log level before logging the message.
/// In WASM environments, it uses `host_log`. In non-WASM environments, it uses the `log` crate.
///
/// # Arguments
///
/// * `level` - The log level of the message.
/// * `message` - The message to log.
pub fn log(level: LogLevel, message: &str) {
    if level.as_i32() >= LOG_LEVEL.load(Ordering::Relaxed) {
        match level {
            LogLevel::Debug => log_debug(message),
            LogLevel::Info => log_info(message),
            LogLevel::Warn => log_warn(message),
            LogLevel::Error => log_error(message),
            LogLevel::None => {}
        }
    }
}

/// Log a debug message.
#[cfg(feature = "wasm")]
pub fn log_debug(message: &str) {
    host_log(LOG_LEVEL_DEBUG, message);
}

#[cfg(not(feature = "wasm"))]
pub fn log_debug(message: &str) {
    debug!("{}", message);
}

/// Log an info message.
#[cfg(feature = "wasm")]
pub fn log_info(message: &str) {
    host_log(LOG_LEVEL_INFO, message);
}

#[cfg(not(feature = "wasm"))]
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log a warning message.
#[cfg(feature = "wasm")]
pub fn log_warn(message: &str) {
    host_log(LOG_LEVEL_WARN, message);
}

#[cfg(not(feature = "wasm"))]
pub fn log_warn(message: &str) {
    warn!("{}", message);
}

/// Log an error message.
#[cfg(feature = "wasm")]
pub fn log_error(message: &str) {
    host_log(LOG_LEVEL_ERROR, message);
}

#[cfg(not(feature = "wasm"))]
pub fn log_error(message: &str) {
    error!("{}", message);
}
