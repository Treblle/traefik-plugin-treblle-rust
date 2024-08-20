use regex::Regex;

pub struct RouteBlacklist {
    patterns: Vec<Regex>,
}

impl RouteBlacklist {
    pub fn new(blacklist: &[String]) -> Self {
        let patterns = blacklist
            .iter()
            .map(|pattern| Regex::new(pattern).unwrap())
            .collect();
        RouteBlacklist { patterns }
    }

    pub fn is_blacklisted(&self, url: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(url))
    }
}
