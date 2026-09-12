use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use serde_serializers::{duration, size};

use super::routes::{ProxyConfig, RouteConfig};
use super::{CacheConfig, ChainTypesConfig, MetricsConfig, MonitoringConfig, RetryConfig};

fn default_request_limit() -> usize {
    32 * 1024 * 1024
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequestConfig {
    #[serde(default = "default_request_limit", deserialize_with = "size::deserialize")]
    pub limit: usize,
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HeadersConfig {
    pub forward: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub address: String,
    pub metrics: MetricsConfig,
    #[serde(skip)]
    pub chains: Option<ChainsConfig>,
    #[serde(skip)]
    pub routes: Option<RoutesConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChainsConfig {
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub chain_types: ChainTypesConfig,
    pub monitoring: MonitoringConfig,
    pub retry: RetryConfig,
    pub request: RequestConfig,
    pub headers: HeadersConfig,
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoutesConfig {
    pub request: RequestConfig,
    pub headers: HeadersConfig,
    pub retry: RetryConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    pub routes: HashMap<String, RouteConfig>,
    pub proxies: Option<HashMap<String, ProxyConfig>>,
}
