use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use config::{Config, ConfigError, Environment, File};
use primitives::{Chain, NodeCheckProfile};
use serde::Deserialize;
use serde_serializers::duration;

mod allowlist;
mod cache;
mod chain_types;
mod domain;
mod metrics;
mod url;

pub use allowlist::AllowlistConfig;
pub use cache::CacheConfig;
pub(crate) use cache::ChainCacheRules;
pub use chain_types::ChainTypesConfig;
pub use domain::ChainConfig;
pub use metrics::MetricsConfig;
pub use url::{Override, Url};

pub(crate) fn path_without_query(path: &str) -> &str {
    path.split_once('?').map_or(path, |(path, _)| path)
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeMonitoringConfig {
    pub enabled: bool,
    #[serde(default)]
    pub profile: NodeCheckProfile,
    #[serde(deserialize_with = "duration::deserialize")]
    pub interval: Duration,
    pub trigger: FailureTriggerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FailureTriggerConfig {
    pub failures: usize,
    pub rate: u8,
    #[serde(deserialize_with = "duration::deserialize")]
    pub window: Duration,
    #[serde(default, deserialize_with = "duration::deserialize_option")]
    pub latency: Option<Duration>,
}

impl FailureTriggerConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.failures == 0 {
            return Err(ConfigError::Message("monitoring.trigger.failures must be greater than zero".to_string()));
        }
        if !(1..=100).contains(&self.rate) {
            return Err(ConfigError::Message("monitoring.trigger.rate must be between 1 and 100".to_string()));
        }
        if self.window.is_zero() {
            return Err(ConfigError::Message("monitoring.trigger.window must be greater than zero".to_string()));
        }
        if self.latency.is_some_and(|latency| latency.is_zero()) {
            return Err(ConfigError::Message("monitoring.trigger.latency must be greater than zero".to_string()));
        }
        Ok(())
    }
}

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
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: usize,
    pub errors: ErrorMatcherConfig,
}

impl RetryConfig {
    pub fn effective_max_attempts(&self, urls_count: usize) -> usize {
        if self.max_attempts == 0 { urls_count } else { self.max_attempts.min(urls_count) }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::normalize_matcher;
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
    fn test_matches_message_case_insensitive_without_normalize() {
        let config = testkit::retry_config(true, vec![], vec!["RATE LIMIT"]);
        assert!(config.errors.matches_message("rate limit exceeded"));
    }

    #[test]
    fn test_normalize_patterns() {
        let mut config = testkit::retry_config(true, vec![], vec![" Rate Limit ", "", "rate limit"]);
        normalize_matcher(&mut config.errors);
        assert_eq!(config.errors.error_messages, vec!["rate limit".to_string()]);
        assert!(config.errors.matches_message("RATE LIMIT EXCEEDED"));
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

    #[test]
    fn test_failure_trigger_validation() {
        let mut trigger = testkit::monitoring_config().trigger;
        assert!(trigger.validate().is_ok());

        trigger.failures = 0;
        assert!(trigger.validate().is_err());
        trigger.failures = 1;

        trigger.rate = 0;
        assert!(trigger.validate().is_err());
        trigger.rate = 101;
        assert!(trigger.validate().is_err());
        trigger.rate = 1;

        trigger.window = Duration::ZERO;
        assert!(trigger.validate().is_err());

        trigger.window = Duration::from_secs(60);
        trigger.latency = Some(Duration::ZERO);
        assert!(trigger.validate().is_err());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequestConfig {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HeadersConfig {
    pub forward: Vec<String>,
    #[serde(default)]
    pub domains: HashMap<String, Vec<String>>,
}

impl HeadersConfig {
    pub fn get_domain_headers(&self, host: &str) -> Option<&[String]> {
        self.domains.get(host).map(Vec::as_slice)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: MetricsConfig,
    pub cache: CacheConfig,
    #[serde(default)]
    pub chain_types: ChainTypesConfig,
    pub monitoring: NodeMonitoringConfig,
    pub retry: RetryConfig,
    pub request: RequestConfig,
    pub headers: HeadersConfig,
    pub jwt: JwtConfig,
    pub webhook: WebhookConfig,
}

impl NodeConfig {
    fn normalize(&mut self) {
        normalize_matcher(&mut self.retry.errors);
    }
}

fn normalize_matcher(matcher: &mut ErrorMatcherConfig) {
    normalize_error_messages(&mut matcher.error_messages);
}

fn normalize_error_messages(messages: &mut Vec<String>) {
    let mut seen = HashSet::with_capacity(messages.len());
    let mut normalized = Vec::with_capacity(messages.len());

    for message in messages.drain(..) {
        let value = message.trim().to_ascii_lowercase();
        if value.is_empty() {
            continue;
        }
        if seen.insert(value.clone()) {
            normalized.push(value);
        }
    }

    *messages = normalized;
}

#[derive(Debug, Deserialize)]
struct ChainsFile {
    chains: Vec<ChainConfig>,
}

pub fn load_config() -> Result<(NodeConfig, HashMap<Chain, ChainConfig>), ConfigError> {
    let current_dir = env::current_dir().unwrap();

    let base_dir = if current_dir.join("config.yml").exists() {
        current_dir
    } else {
        current_dir.join("apps/dynode")
    };

    let mut config: NodeConfig = Config::builder()
        .add_source(File::from(base_dir.join("config.yml")))
        .add_source(File::from(base_dir.join("cache.yml")))
        .add_source(Environment::default().separator("_"))
        .build()?
        .try_deserialize()?;
    config.normalize();
    config.monitoring.trigger.validate()?;

    let chains = find_chain_files(&base_dir)
        .into_iter()
        .map(|path| Config::builder().add_source(File::from(path)).build()?.try_deserialize::<ChainsFile>())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|cf| cf.chains)
        .filter(|config| !config.chain.is_disabled())
        .map(|c| (c.chain, c))
        .collect();

    Ok((config, chains))
}

fn find_chain_files(base_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(base_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("chains") && name.ends_with(".yml"))
        })
        .collect();

    files.sort();
    files
}
