use std::collections::HashMap;
use std::num::NonZeroU32;
use std::time::Duration;

use serde::Deserialize;
use serde_serializers::duration;

use super::path::PathAllowlist;
use super::{CacheRule, HeadersConfig};

#[derive(Clone, Debug, Deserialize)]
pub struct RetryOverride {
    pub statuses: Vec<u16>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProxyConfig {
    pub url: String,
    pub health: ProxyHealthConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProxyHealthConfig {
    pub url: String,
    #[serde(deserialize_with = "duration::deserialize")]
    pub interval: Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RouteConfig {
    pub group: String,
    pub selection: Selection,
    pub headers: Option<HeadersConfig>,
    #[serde(default)]
    pub allowlist: PathAllowlist,
    #[serde(default)]
    pub(crate) cache: Vec<CacheRule>,
    pub rate: Option<RateConfig>,
    pub retry: Option<RetryOverride>,
    pub endpoints: Vec<EndpointConfig>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct RateConfig {
    pub requests: NonZeroU32,
    #[serde(deserialize_with = "duration::deserialize")]
    pub period: Duration,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Selection {
    Ordered,
    Random,
    RoundRobin,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EndpointConfig {
    pub name: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub query: Option<HashMap<String, String>>,
    pub proxy: Option<String>,
}
