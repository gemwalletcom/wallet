use std::collections::HashMap;
use std::num::NonZeroU32;
use std::time::Duration;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
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
    #[serde(rename = "group", deserialize_with = "deserialize_groups")]
    pub groups: Vec<String>,
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

#[derive(Deserialize)]
#[serde(untagged)]
enum Groups {
    One(String),
    Many(Vec<String>),
}

fn deserialize_groups<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let groups = match Groups::deserialize(deserializer)? {
        Groups::One(group) => vec![group],
        Groups::Many(groups) => groups,
    };
    if groups.is_empty() {
        return Err(D::Error::custom("route must configure at least one group"));
    }
    Ok(groups)
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
