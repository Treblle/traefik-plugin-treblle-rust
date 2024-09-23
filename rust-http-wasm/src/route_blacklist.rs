//! Route blacklist module for the Treblle middleware.
//!
//! This module provides functionality to check if a given URL is blacklisted.

use regex::Regex;

/// Represents a collection of blacklisted routes.
#[derive(Clone)]
pub struct RouteBlacklist {
    patterns: Vec<Regex>,
}

impl RouteBlacklist {
    /// Creates a new `RouteBlacklist` instance.
    ///
    /// # Arguments
    ///
    /// * `blacklist` - A slice of strings representing regex patterns for blacklisted routes.
    ///
    /// # Panics
    ///
    /// Panics if any of the provided patterns are invalid regular expressions.
    pub fn new(blacklist: &[String]) -> Self {
        let patterns = blacklist
            .iter()
            .map(|pattern| Regex::new(pattern).expect("Invalid regex pattern in blacklist"))
            .collect();
        RouteBlacklist { patterns }
    }

    /// Checks if a given URL is blacklisted.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check against the blacklist.
    ///
    /// # Returns
    ///
    /// Returns `true` if the URL matches any of the blacklisted patterns, `false` otherwise.
    pub fn is_blacklisted(&self, url: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_blacklist() {
        let blacklist = RouteBlacklist::new(&[
            r"^/api/internal/.*$".to_string(),
            r"^/health$".to_string(),
        ]);

        assert!(blacklist.is_blacklisted("/api/internal/users"));
        assert!(blacklist.is_blacklisted("/health"));
        assert!(!blacklist.is_blacklisted("/api/public/users"));
        assert!(!blacklist.is_blacklisted("/healthcheck"));
    }

    #[test]
    #[should_panic(expected = "Invalid regex pattern in blacklist")]
    fn test_invalid_regex() {
        RouteBlacklist::new(&["[invalid regex".to_string()]);
    }
}