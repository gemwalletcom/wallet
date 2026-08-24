mod loader;

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::time::Duration;

use serde::Deserialize;
use serde_serializers::{duration, size};

#[derive(Debug, Deserialize)]
pub(crate) struct EgressConfig {
    pub address: IpAddr,
    pub port: u16,
    pub callers: HashMap<String, CallerConfig>,
    pub headers: HeadersConfig,
    pub request: RequestConfig,
    pub retry: RetryConfig,
    pub proxies: Option<HashMap<String, ProxyConfig>>,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallerConfig {
    pub key: String,
    pub groups: HashSet<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct HeadersConfig {
    pub forward: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RequestConfig {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    #[serde(deserialize_with = "size::deserialize")]
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RetryConfig {
    #[serde(deserialize_with = "duration::deserialize")]
    pub cooldown: Duration,
    pub statuses: Vec<u16>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RetryOverride {
    pub statuses: Vec<u16>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProxyConfig {
    pub url: String,
    pub health: ProxyHealthConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ProxyHealthConfig {
    pub url: String,
    #[serde(deserialize_with = "duration::deserialize")]
    pub interval: Duration,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RouteConfig {
    pub group: String,
    pub service: String,
    pub selection: Selection,
    pub retry: Option<RetryOverride>,
    pub endpoints: Vec<EndpointConfig>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Selection {
    Ordered,
    Random,
    RoundRobin,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EndpointConfig {
    pub name: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub query: Option<HashMap<String, String>>,
    pub proxy: Option<String>,
}
