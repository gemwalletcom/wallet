use std::time::Duration;

use serde::Deserialize;
use serde_serializers::duration;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ErrorMatcherConfig {
    pub status: Option<u16>,
    pub message: Option<String>,
}

impl ErrorMatcherConfig {
    pub fn matches(&self, status: u16, message: Option<&str>) -> bool {
        if self.status.is_some_and(|expected| expected != status) {
            return false;
        }

        match self.message.as_deref() {
            Some(pattern) => {
                let pattern = pattern.trim();
                !pattern.is_empty() && message.is_some_and(|message| message.to_ascii_lowercase().contains(&pattern.to_ascii_lowercase()))
            }
            None => self.status.is_some(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: usize,
    pub errors: Vec<ErrorMatcherConfig>,
    #[serde(deserialize_with = "duration::deserialize")]
    pub cooldown: Duration,
    pub statuses: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_attempts: 0,
            errors: Vec::new(),
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
    use super::*;

    #[test]
    fn test_matches_message() {
        let rule = ErrorMatcherConfig {
            message: Some(" Rate Limit ".to_string()),
            ..ErrorMatcherConfig::default()
        };

        assert!(rule.matches(200, Some("rate limit exceeded")));
        assert!(rule.matches(400, Some("Rate Limit Exceeded")));
        assert!(!rule.matches(400, Some("internal server error")));
        assert!(!rule.matches(400, Some("")));
        assert!(!rule.matches(400, None));
    }

    #[test]
    fn test_matches_empty_rule() {
        let rule = ErrorMatcherConfig::default();
        assert!(!rule.matches(400, Some("error")));

        let rule = ErrorMatcherConfig {
            status: Some(400),
            message: Some(" ".to_string()),
        };
        assert!(!rule.matches(400, Some("error")));
    }

    #[test]
    fn test_matches_status() {
        let rule = ErrorMatcherConfig {
            status: Some(429),
            ..ErrorMatcherConfig::default()
        };
        assert!(rule.matches(429, None));
        assert!(rule.matches(429, Some("any error")));
        assert!(!rule.matches(500, None));
    }

    #[test]
    fn test_matches_status_and_message() {
        let rule = ErrorMatcherConfig {
            status: Some(400),
            message: Some("502 Bad Gateway".to_string()),
        };
        assert!(rule.matches(400, Some("502 Bad Gateway invalid character '<'")));
        assert!(!rule.matches(200, Some("502 Bad Gateway invalid character '<'")));
        assert!(!rule.matches(400, Some("invalid transaction")));
        assert!(!rule.matches(400, None));
    }

    #[test]
    fn test_effective_max_attempts() {
        let config_zero = RetryConfig::mock();
        assert_eq!(config_zero.effective_max_attempts(5), 5);
        assert_eq!(config_zero.effective_max_attempts(10), 10);

        let config_limited = RetryConfig { max_attempts: 3, ..RetryConfig::mock() };
        assert_eq!(config_limited.effective_max_attempts(5), 3);
        assert_eq!(config_limited.effective_max_attempts(2), 2);
    }
}
