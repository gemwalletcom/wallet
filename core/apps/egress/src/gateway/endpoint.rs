use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use config::ConfigError;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use url::Url;

use super::BoxError;
use super::proxy::OutboundProxy;
use crate::config::EndpointConfig;

pub(super) struct Endpoint {
    pub(super) name: String,
    pub(super) url: Url,
    pub(super) client: Client,
    pub(super) query: HashMap<String, String>,
    proxy_available: Option<Arc<AtomicBool>>,
    headers: HeaderMap,
}

impl Endpoint {
    pub(super) fn new(config: EndpointConfig, direct_client: &Client, proxies: &HashMap<String, OutboundProxy>) -> Result<Self, BoxError> {
        let (client, proxy_available) = match config.proxy.as_ref() {
            Some(name) => {
                let proxy = proxies.get(name).ok_or_else(|| ConfigError::Message(format!("unknown proxy: {name}")))?;
                (proxy.client.clone(), Some(proxy.availability()))
            }
            None => (direct_client.clone(), None),
        };
        let headers = config
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| Ok((HeaderName::from_bytes(name.as_bytes())?, HeaderValue::from_str(&value)?)))
            .collect::<Result<HeaderMap, BoxError>>()?;
        Ok(Self {
            name: config.name,
            url: Url::parse(&config.url)?,
            client,
            query: config.query.unwrap_or_default(),
            proxy_available,
            headers,
        })
    }

    pub(super) fn is_available(&self) -> bool {
        self.proxy_available.as_ref().is_none_or(|available| available.load(Ordering::Relaxed))
    }

    pub(super) fn request_headers(&self, inbound: &HeaderMap, allowed: &HashSet<HeaderName>) -> HeaderMap {
        let mut headers = filter_headers(inbound, allowed);
        for (name, value) in &self.headers {
            headers.insert(name.clone(), value.clone());
        }
        headers
    }

    pub(super) fn key(&self, group: &str, service: &str, path: &str) -> String {
        format!("{group}:{service}:{}:{path}", self.name)
    }
}

pub(super) fn filter_headers(headers: &HeaderMap, allowed: &HashSet<HeaderName>) -> HeaderMap {
    headers
        .iter()
        .filter(|(name, _)| allowed.contains(*name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_headers() {
        let inbound = HeaderMap::from_iter([
            (reqwest::header::AUTHORIZATION, HeaderValue::from_static("Bearer ingress")),
            (reqwest::header::ACCEPT, HeaderValue::from_static("application/json")),
        ]);
        let endpoint = Endpoint {
            name: "key_1".to_string(),
            url: Url::parse("https://tonapi.io").unwrap(),
            client: Client::new(),
            query: HashMap::new(),
            proxy_available: None,
            headers: HeaderMap::from_iter([(reqwest::header::AUTHORIZATION, HeaderValue::from_static("Bearer upstream"))]),
        };
        let result = endpoint.request_headers(&inbound, &HashSet::from([reqwest::header::ACCEPT]));
        assert_eq!(result.get(reqwest::header::AUTHORIZATION).unwrap(), "Bearer upstream");
        assert_eq!(result.get(reqwest::header::ACCEPT).unwrap(), "application/json");
    }
}
