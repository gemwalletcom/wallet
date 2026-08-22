mod access;
mod endpoint;
mod proxy;
mod route;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::{Duration, Instant};

use gem_tracing::path;
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderName, RETRY_AFTER};
use rocket::http::Status;
use tokio::sync::RwLock;

use access::AccessLog;
use endpoint::{Endpoint, filter_headers};
use proxy::{OutboundProxy, build_client};
use route::{Route, match_route};

use crate::config::EgressConfig;
use crate::metrics::Metrics;

type BoxError = Box<dyn Error + Send + Sync>;

pub(crate) struct Gateway {
    routes: Vec<Route>,
    proxies: HashMap<String, OutboundProxy>,
    cooldowns: RwLock<HashMap<String, Instant>>,
    cooldown: Duration,
    forward_headers: HashSet<HeaderName>,
    metrics: Metrics,
}

pub(crate) struct GatewayResponse {
    pub(super) status: u16,
    pub(super) headers: HeaderMap,
    pub(super) body: Vec<u8>,
}

pub(crate) struct GatewayError {
    pub(super) status: Status,
    pub(super) message: String,
}

impl Gateway {
    pub(crate) fn new(config: EgressConfig, metrics: Metrics) -> Result<Self, BoxError> {
        let EgressConfig {
            headers,
            request,
            retry,
            proxies,
            routes,
            ..
        } = config;
        let direct_client = build_client(request.timeout, None)?;
        let proxies = proxies
            .unwrap_or_default()
            .into_iter()
            .map(|(name, config)| Ok((name, OutboundProxy::new(config, request.timeout)?)))
            .collect::<Result<HashMap<_, _>, reqwest::Error>>()?;
        let routes = routes
            .into_iter()
            .map(|route| Route::new(route, &retry.statuses, &direct_client, &proxies))
            .collect::<Result<Vec<_>, BoxError>>()?;
        let forward_headers = headers
            .forward
            .iter()
            .map(|header| HeaderName::from_bytes(header.as_bytes()))
            .collect::<Result<HashSet<_>, _>>()?;
        for name in proxies.keys() {
            metrics.set_proxy_available(name, false);
        }

        Ok(Self {
            routes,
            proxies,
            cooldowns: RwLock::new(HashMap::new()),
            cooldown: retry.cooldown,
            forward_headers,
            metrics,
        })
    }

    pub(crate) fn start_health_checks(&self) {
        for (name, proxy) in &self.proxies {
            proxy.start_health_check(name.clone(), self.metrics.clone());
        }
    }

    pub(crate) async fn forward(&self, method: Method, uri: &str, headers: &HeaderMap, body: Vec<u8>) -> Result<GatewayResponse, GatewayError> {
        let Some(route_match) = match_route(&self.routes, uri) else {
            let uri = path::redact(uri);
            let access = AccessLog::new(&method, &uri);
            access.request(None);
            access.unavailable(None, Status::NotFound.code, "route");
            return Err(GatewayError::new(Status::NotFound, "route not found"));
        };
        let route = route_match.route;
        let path = route_match.path();
        let access = AccessLog::new(&method, &path);
        access.request(Some(route));
        let mut candidates = self.available_endpoints(route).await;
        if candidates.is_empty() {
            access.unavailable(Some(route), Status::ServiceUnavailable.code, "endpoints");
            return Err(GatewayError::new(Status::ServiceUnavailable, "no healthy endpoint is available"));
        }
        route.prioritize_endpoints(&mut candidates);

        let mut last_response = None;
        for (position, endpoint_index) in candidates.iter().enumerate() {
            let endpoint = &route.endpoints[*endpoint_index];
            let remote_host = endpoint.url.host_str().unwrap_or("none");
            let target = match route_match.target_url(endpoint) {
                Ok(target) => target,
                Err(error) => {
                    access.response(route, &endpoint.name, remote_host, Status::BadRequest.code);
                    return Err(GatewayError::new(Status::BadRequest, error.to_string()));
                }
            };
            let request_headers = endpoint.request_headers(headers, &self.forward_headers);
            let result = endpoint.client.request(method.clone(), target).headers(request_headers).body(body.clone()).send().await;
            let has_next = position + 1 < candidates.len();

            match result {
                Ok(response) => {
                    let status = response.status();
                    let retry_after = retry_after(response.headers());
                    let response_headers = filter_headers(response.headers(), &self.forward_headers);
                    let response_body = match response.bytes().await {
                        Ok(bytes) => bytes.to_vec(),
                        Err(_) => {
                            access.upstream_failed(route, &endpoint.name, remote_host, "response_body");
                            self.record_failure(route, endpoint, &path, "response_body", self.cooldown).await;
                            if has_next {
                                continue;
                            }
                            break;
                        }
                    };
                    self.metrics.record_request(&route.group, &route.service, &endpoint.name, &path, status.as_u16());
                    let response = GatewayResponse {
                        status: status.as_u16(),
                        headers: response_headers,
                        body: response_body,
                    };
                    if route.should_retry(status) {
                        self.record_failure(route, endpoint, &path, &status.as_u16().to_string(), retry_after.unwrap_or(self.cooldown))
                            .await;
                        if has_next {
                            access.failover(route, &endpoint.name, remote_host, status.as_u16());
                            last_response = Some((response, endpoint));
                            continue;
                        }
                    } else {
                        self.clear_failure(route, endpoint).await;
                    }
                    access.response(route, &endpoint.name, remote_host, status.as_u16());
                    return Ok(response);
                }
                Err(_) => {
                    access.upstream_failed(route, &endpoint.name, remote_host, "transport");
                    self.record_failure(route, endpoint, &path, "transport", self.cooldown).await;
                    if !has_next {
                        break;
                    }
                }
            }
        }

        if let Some((response, endpoint)) = last_response {
            access.response(route, &endpoint.name, endpoint.url.host_str().unwrap_or("none"), response.status);
            return Ok(response);
        }
        access.unavailable(Some(route), Status::BadGateway.code, "upstreams");
        Err(GatewayError::new(Status::BadGateway, "all upstream requests failed"))
    }

    async fn available_endpoints(&self, route: &Route) -> Vec<usize> {
        let now = Instant::now();
        let mut cooldowns = self.cooldowns.write().await;
        cooldowns.retain(|_, until| *until > now);
        route
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, endpoint)| endpoint.is_available())
            .filter(|(_, endpoint)| !cooldowns.contains_key(&endpoint.key(&route.group, &route.service)))
            .map(|(index, _)| index)
            .collect()
    }

    async fn record_failure(&self, route: &Route, endpoint: &Endpoint, path: &str, reason: &str, cooldown: Duration) {
        self.cooldowns.write().await.insert(endpoint.key(&route.group, &route.service), Instant::now() + cooldown);
        self.metrics.record_failover(&route.group, &route.service, &endpoint.name, path, reason);
    }

    async fn clear_failure(&self, route: &Route, endpoint: &Endpoint) {
        self.cooldowns.write().await.remove(&endpoint.key(&route.group, &route.service));
    }
}

impl GatewayError {
    pub(crate) fn new(status: Status, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let duration = Duration::from_secs(headers.get(RETRY_AFTER)?.to_str().ok()?.parse::<u64>().ok()?);
    Instant::now().checked_add(duration)?;
    Some(duration)
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderValue;

    use super::*;

    #[test]
    fn test_retry_after() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("120"));
        assert_eq!(retry_after(&headers), Some(Duration::from_secs(120)));

        headers.insert(RETRY_AFTER, HeaderValue::from_static("invalid"));
        assert_eq!(retry_after(&headers), None);

        headers.insert(RETRY_AFTER, HeaderValue::from_static("18446744073709551615"));
        assert_eq!(retry_after(&headers), None);
    }
}
