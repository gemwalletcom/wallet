use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use config::ConfigError;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, RETRY_AFTER};
use reqwest::{Client, Method};
use tokio::sync::Mutex;
use tokio::time::{Instant as TokioInstant, sleep};
use url::Url;

use super::proxy::OutboundProxy;
use super::{BoxError, GatewayResponse};
use crate::config::{EndpointConfig, RateConfig};

pub(super) struct Endpoint {
    pub(super) name: String,
    pub(super) url: Url,
    pub(super) host: String,
    pub(super) query: HashMap<String, String>,
    client: Client,
    proxy_available: Option<Arc<AtomicBool>>,
    headers: HeaderMap,
    throttle: Option<Throttle>,
}

struct Throttle {
    interval: Duration,
    next: Mutex<TokioInstant>,
}

impl Endpoint {
    pub(super) fn new(config: EndpointConfig, rate: Option<RateConfig>, direct_client: &Client, proxies: &HashMap<String, OutboundProxy>) -> Result<Self, BoxError> {
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
        let url = Url::parse(&config.url)?;
        let host = url.host_str().ok_or_else(|| ConfigError::Message("endpoint URL must include a host".into()))?.to_string();
        Ok(Self {
            name: config.name,
            url,
            host,
            client,
            query: config.query.unwrap_or_default(),
            proxy_available,
            headers,
            throttle: rate.map(Throttle::new),
        })
    }

    pub(super) fn is_available(&self) -> bool {
        self.proxy_available.as_ref().is_none_or(|available| available.load(Ordering::Relaxed))
    }

    pub(super) async fn throttle(&self) -> Option<Duration> {
        match &self.throttle {
            Some(throttle) => Some(throttle.wait().await),
            None => None,
        }
    }

    fn request_headers(&self, inbound: &HeaderMap, forward_headers: &HashSet<HeaderName>) -> HeaderMap {
        let mut headers = Self::filter_headers(inbound, forward_headers);
        for (name, value) in &self.headers {
            headers.insert(name.clone(), value.clone());
        }
        headers
    }

    pub(super) async fn send(
        &self,
        method: &Method,
        url: Url,
        inbound_headers: &HeaderMap,
        forward_headers: &HashSet<HeaderName>,
        body: Vec<u8>,
    ) -> Result<(GatewayResponse, Option<Duration>), &'static str> {
        let response = self
            .client
            .request(method.clone(), url)
            .headers(self.request_headers(inbound_headers, forward_headers))
            .body(body)
            .send()
            .await
            .map_err(|_| "transport")?;
        let status = response.status().as_u16();
        let retry_after = Self::retry_after(response.headers());
        let headers = Self::filter_headers(response.headers(), forward_headers);
        let body = response.bytes().await.map_err(|_| "response_body")?.to_vec();
        Ok((GatewayResponse { status, headers, body }, retry_after))
    }

    pub(super) fn cooldown_key(&self, group: &str, service: &str, path: &str) -> String {
        format!("{group}:{service}:{}:{path}", self.name)
    }

    fn filter_headers(headers: &HeaderMap, forward_headers: &HashSet<HeaderName>) -> HeaderMap {
        headers
            .iter()
            .filter(|(name, _)| forward_headers.contains(*name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect()
    }

    fn retry_after(headers: &HeaderMap) -> Option<Duration> {
        let duration = Duration::from_secs(headers.get(RETRY_AFTER)?.to_str().ok()?.parse::<u64>().ok()?);
        Instant::now().checked_add(duration)?;
        Some(duration)
    }
}

impl Throttle {
    fn new(rate: RateConfig) -> Self {
        Self {
            interval: rate.period / rate.requests.get(),
            next: Mutex::new(TokioInstant::now()),
        }
    }

    async fn wait(&self) -> Duration {
        let mut next = self.next.lock().await;
        let wait = next.saturating_duration_since(TokioInstant::now());
        sleep(wait).await;
        *next = TokioInstant::now() + self.interval;
        wait
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue};

    use super::*;

    #[test]
    fn test_request_headers() {
        let inbound = HeaderMap::from_iter([
            (AUTHORIZATION, HeaderValue::from_static("Bearer ingress")),
            (ACCEPT, HeaderValue::from_static("application/json")),
        ]);
        let endpoint = Endpoint {
            name: "key_1".to_string(),
            url: Url::parse("https://tonapi.io").unwrap(),
            host: "tonapi.io".to_string(),
            client: Client::new(),
            query: HashMap::new(),
            proxy_available: None,
            headers: HeaderMap::from_iter([(AUTHORIZATION, HeaderValue::from_static("Bearer upstream"))]),
            throttle: None,
        };
        let result = endpoint.request_headers(&inbound, &HashSet::from([ACCEPT]));
        assert_eq!(result.get(AUTHORIZATION).unwrap(), "Bearer upstream");
        assert_eq!(result.get(ACCEPT).unwrap(), "application/json");
    }

    #[test]
    fn test_retry_after() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("120"));
        assert_eq!(Endpoint::retry_after(&headers), Some(Duration::from_mins(2)));

        headers.insert(RETRY_AFTER, HeaderValue::from_static("invalid"));
        assert_eq!(Endpoint::retry_after(&headers), None);

        headers.insert(RETRY_AFTER, HeaderValue::from_static("18446744073709551615"));
        assert_eq!(Endpoint::retry_after(&headers), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_throttle_pacing() {
        let throttle = Throttle::new(RateConfig {
            requests: NonZeroU32::new(5).unwrap(),
            period: Duration::from_secs(1),
        });
        assert_eq!(throttle.wait().await, Duration::ZERO);
        assert_eq!(throttle.wait().await, Duration::from_millis(200));
        assert_eq!(throttle.wait().await, Duration::from_millis(200));
    }
}
