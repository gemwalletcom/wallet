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
    cooldowns: RwLock<HashMap<String, Cooldown>>,
    cooldown: Duration,
    forward_headers: HashSet<HeaderName>,
    metrics: Metrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Failure {
    status: u16,
    reason: &'static str,
}

struct Cooldown {
    until: Instant,
    failure: Failure,
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
        let mut candidates = self.available_endpoints(route, &path).await.map_err(|failure| {
            access.unavailable(Some(route), failure.status, failure.reason);
            self.metrics.record_response(&route.group, &route.service, &path, failure.status);
            GatewayError::new(Status::from_code(failure.status).unwrap_or(Status::ServiceUnavailable), "no endpoint is available")
        })?;
        route.prioritize_endpoints(&mut candidates);

        let mut last_response = None;
        let mut pending_failover: Option<(usize, u16, String)> = None;
        for endpoint_index in candidates {
            let endpoint = &route.endpoints[endpoint_index];
            if !self.endpoint_available(route, endpoint, &path).await {
                continue;
            }
            if let Some((failed_index, status, reason)) = pending_failover.take() {
                let failed = &route.endpoints[failed_index];
                access.failover(route, &failed.name, failed.url.host_str().unwrap_or("none"), status);
                self.metrics.record_failover(&route.group, &route.service, &failed.name, &path, &reason);
            }
            let remote_host = endpoint.url.host_str().unwrap_or("none");
            let target = match route_match.target_url(endpoint) {
                Ok(target) => target,
                Err(error) => {
                    access.response(route, &endpoint.name, remote_host, Status::BadRequest.code);
                    self.metrics.record_response(&route.group, &route.service, &path, Status::BadRequest.code);
                    return Err(GatewayError::new(Status::BadRequest, error.to_string()));
                }
            };
            let request_headers = endpoint.request_headers(headers, &self.forward_headers);
            let result = endpoint.client.request(method.clone(), target).headers(request_headers).body(body.clone()).send().await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    let retry_after = retry_after(response.headers());
                    let response_headers = filter_headers(response.headers(), &self.forward_headers);
                    let response_body = match response.bytes().await {
                        Ok(bytes) => bytes.to_vec(),
                        Err(_) => {
                            access.upstream_failed(route, &endpoint.name, remote_host, "response_body");
                            self.record_failure(
                                route,
                                endpoint,
                                &path,
                                Failure {
                                    status: Status::ServiceUnavailable.code,
                                    reason: "response_body",
                                },
                                self.cooldown,
                            )
                            .await;
                            pending_failover = Some((endpoint_index, Status::BadGateway.code, "response_body".to_string()));
                            continue;
                        }
                    };
                    self.metrics.record_request(&route.group, &route.service, &endpoint.name, &path, status.as_u16());
                    let response = GatewayResponse {
                        status: status.as_u16(),
                        headers: response_headers,
                        body: response_body,
                    };
                    if route.should_retry(status) {
                        self.record_failure(
                            route,
                            endpoint,
                            &path,
                            Failure {
                                status: status.as_u16(),
                                reason: "cooldown",
                            },
                            retry_after.unwrap_or(self.cooldown),
                        )
                        .await;
                        last_response = Some((response, endpoint_index));
                        pending_failover = Some((endpoint_index, status.as_u16(), status.as_u16().to_string()));
                        continue;
                    }
                    access.response(route, &endpoint.name, remote_host, status.as_u16());
                    self.metrics.record_response(&route.group, &route.service, &path, status.as_u16());
                    return Ok(response);
                }
                Err(_) => {
                    access.upstream_failed(route, &endpoint.name, remote_host, "transport");
                    self.record_failure(
                        route,
                        endpoint,
                        &path,
                        Failure {
                            status: Status::ServiceUnavailable.code,
                            reason: "transport",
                        },
                        self.cooldown,
                    )
                    .await;
                    pending_failover = Some((endpoint_index, Status::BadGateway.code, "transport".to_string()));
                }
            }
        }

        if let Some((response, endpoint_index)) = last_response {
            let endpoint = &route.endpoints[endpoint_index];
            access.response(route, &endpoint.name, endpoint.url.host_str().unwrap_or("none"), response.status);
            self.metrics.record_response(&route.group, &route.service, &path, response.status);
            return Ok(response);
        }
        let failure = self.available_endpoints(route, &path).await.err().unwrap_or(Failure {
            status: Status::ServiceUnavailable.code,
            reason: "upstream",
        });
        access.unavailable(Some(route), failure.status, failure.reason);
        self.metrics.record_response(&route.group, &route.service, &path, failure.status);
        Err(GatewayError::new(
            Status::from_code(failure.status).unwrap_or(Status::ServiceUnavailable),
            "all upstream requests failed",
        ))
    }

    async fn available_endpoints(&self, route: &Route, path: &str) -> Result<Vec<usize>, Failure> {
        let now = Instant::now();
        let mut cooldowns = self.cooldowns.write().await;
        cooldowns.retain(|_, cooldown| cooldown.until > now);
        let mut failures = Vec::new();
        let endpoints = route
            .endpoints
            .iter()
            .enumerate()
            .filter_map(|(index, endpoint)| {
                if !endpoint.is_available() {
                    failures.push(Failure {
                        status: Status::ServiceUnavailable.code,
                        reason: "proxy",
                    });
                    return None;
                }
                if let Some(cooldown) = cooldowns.get(&endpoint.key(&route.group, &route.service, path)) {
                    failures.push(cooldown.failure);
                    return None;
                }
                Some(index)
            })
            .collect::<Vec<_>>();
        if !endpoints.is_empty() {
            return Ok(endpoints);
        }
        let first = failures.first().copied().unwrap_or(Failure {
            status: Status::ServiceUnavailable.code,
            reason: "configuration",
        });
        if failures.iter().all(|failure| *failure == first) {
            return Err(first);
        }
        if failures.iter().all(|failure| failure.reason == "cooldown") {
            return Err(Failure {
                status: failures.iter().fold(first.status, |status, failure| status.max(failure.status)),
                reason: "cooldown",
            });
        }
        Err(Failure {
            status: Status::ServiceUnavailable.code,
            reason: "mixed",
        })
    }

    async fn endpoint_available(&self, route: &Route, endpoint: &Endpoint, path: &str) -> bool {
        endpoint.is_available()
            && self
                .cooldowns
                .read()
                .await
                .get(&endpoint.key(&route.group, &route.service, path))
                .is_none_or(|cooldown| cooldown.until <= Instant::now())
    }

    async fn record_failure(&self, route: &Route, endpoint: &Endpoint, path: &str, failure: Failure, duration: Duration) {
        self.cooldowns.write().await.insert(
            endpoint.key(&route.group, &route.service, path),
            Cooldown {
                until: Instant::now() + duration,
                failure,
            },
        );
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
    use std::net::{IpAddr, Ipv4Addr};

    use reqwest::header::HeaderValue;

    use super::*;
    use crate::config::{EndpointConfig, HeadersConfig, RequestConfig, RetryConfig, RouteConfig, Selection};

    fn gateway() -> Gateway {
        Gateway::new(
            EgressConfig {
                address: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0,
                headers: HeadersConfig { forward: Vec::new() },
                request: RequestConfig {
                    timeout: Duration::from_secs(1),
                    limit: 1024,
                },
                retry: RetryConfig {
                    cooldown: Duration::from_secs(60),
                    statuses: vec![429],
                },
                proxies: None,
                routes: vec![RouteConfig {
                    group: "indexer".to_string(),
                    service: "blockscout".to_string(),
                    selection: Selection::Ordered,
                    retry: None,
                    endpoints: ["key_1", "key_2"]
                        .map(|name| EndpointConfig {
                            name: name.to_string(),
                            url: "https://api.blockscout.com".to_string(),
                            headers: None,
                            query: None,
                            proxy: None,
                        })
                        .into(),
                }],
            },
            Metrics::new(),
        )
        .unwrap()
    }

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

    #[tokio::test]
    async fn cooldowns_preserve_status_and_path() {
        let gateway = gateway();
        let route = &gateway.routes[0];
        let failure = Failure {
            status: Status::TooManyRequests.code,
            reason: "cooldown",
        };
        for endpoint in &route.endpoints {
            gateway
                .record_failure(route, endpoint, "/api/v2/addresses/:value/token-transfers", failure, gateway.cooldown)
                .await;
        }

        let unavailable = gateway.available_endpoints(route, "/api/v2/addresses/:value/token-transfers").await.unwrap_err();
        assert_eq!(unavailable.status, Status::TooManyRequests.code);
        assert_eq!(unavailable.reason, "cooldown");

        let available = gateway.available_endpoints(route, "/api/v2/addresses/:value/token-balances").await.unwrap();
        assert_eq!(available, vec![0, 1]);

        gateway
            .record_failure(
                route,
                &route.endpoints[0],
                "/mixed",
                Failure {
                    status: Status::Forbidden.code,
                    reason: "cooldown",
                },
                gateway.cooldown,
            )
            .await;
        gateway.record_failure(route, &route.endpoints[1], "/mixed", failure, gateway.cooldown).await;
        let unavailable = gateway.available_endpoints(route, "/mixed").await.unwrap_err();
        assert_eq!(unavailable.status, Status::TooManyRequests.code);
        assert_eq!(unavailable.reason, "cooldown");
    }
}
