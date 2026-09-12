use std::time::Duration;

use serde::Deserialize;
use serde_serializers::duration;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ErrorMatcherConfig {
    pub status_codes: Vec<u16>,
    pub error_messages: Vec<String>,
}

impl ErrorMatcherConfig {
    pub fn matches_status(&self, status: u16) -> bool {
        self.status_codes.contains(&status)
    }

    pub fn matches_message(&self, message: &str) -> bool {
        if message.is_empty() {
            return false;
        }

        let message_lower = message.to_ascii_lowercase();
        self.error_messages.iter().any(|pattern| {
            let pattern = pattern.trim();
            !pattern.is_empty() && message_lower.contains(&pattern.to_ascii_lowercase())
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: usize,
    pub errors: ErrorMatcherConfig,
    #[serde(deserialize_with = "duration::deserialize")]
    pub cooldown: Duration,
    pub statuses: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_attempts: 0,
            errors: ErrorMatcherConfig::default(),
            cooldown: Duration::from_secs(60),
            statuses: vec![403, 429],
        }
    }
}

impl RetryConfig {
    pub fn effective_max_attempts(&self, urls_count: usize) -> usize {
        if self.max_attempts == 0 { urls_count } else { self.max_attempts.min(urls_count) }
    }
}

#[cfg(test)]
mod tests {
    use crate::testkit::config as testkit;

    #[test]
    fn test_should_retry_on_error_message() {
        let config = testkit::retry_config(true, vec![], vec!["daily request limit", "rate limit"]);

        assert!(config.errors.matches_message("daily request limit reached - upgrade your account"));
        assert!(config.errors.matches_message("rate limit exceeded"));
        assert!(config.errors.matches_message("Rate Limit Exceeded"));
        assert!(!config.errors.matches_message("internal server error"));
        assert!(!config.errors.matches_message(""));
    }

    #[test]
    fn test_should_retry_on_error_message_empty() {
        let config = testkit::retry_config(true, vec![], vec![]);

        assert!(!config.errors.matches_message("daily request limit reached"));
    }

    #[test]
    fn test_matches_status() {
        let config = testkit::retry_config(true, vec![401, 403, 429], vec![]);
        assert!(config.errors.matches_status(429));
        assert!(!config.errors.matches_status(500));
    }

    #[test]
    fn test_matches_message_case_and_whitespace() {
        let config = testkit::retry_config(true, vec![], vec![" Rate Limit ", "", "rate limit"]);
        assert!(config.errors.matches_message("rate limit exceeded"));
        assert!(!config.errors.matches_message("unrelated error"));
    }

    #[test]
    fn test_effective_max_attempts() {
        let config_zero = testkit::retry_config(true, vec![], vec![]);
        assert_eq!(config_zero.effective_max_attempts(5), 5);
        assert_eq!(config_zero.effective_max_attempts(10), 10);

        let config_limited = testkit::retry_config_with_attempts(true, 3, vec![], vec![]);
        assert_eq!(config_limited.effective_max_attempts(5), 3);
        assert_eq!(config_limited.effective_max_attempts(2), 2);
    }
}
