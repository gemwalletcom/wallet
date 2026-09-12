use std::collections::HashMap;
use std::time::{Duration, Instant};

use gem_tracing::{info_with_fields, path};
use reqwest::header::{CACHE_CONTROL, HeaderMap, SET_COOKIE, VARY};
use reqwest::{Error as RequestError, Method};
use rocket::http::Status;
use tokio::sync::RwLock;

use super::access::AccessLog;
use super::endpoint::Endpoint;
use super::proxy::{OutboundProxy, build_client};
use super::route::{MatchError, Route, match_route};
use crate::BoxError;
use crate::cache::RequestCache;
use crate::config::RoutesConfig;
use crate::metrics::Metrics;
use crate::proxy::{ProxyResponse, transport};
use crate::response::ProxyError;

pub(crate) struct Gateway {
    routes: HashMap<String, Route>,
    proxies: HashMap<String, OutboundProxy>,
    cooldowns: RwLock<HashMap<String, Cooldown>>,
    cooldown: Duration,
    metrics: Metrics,
    cache: RequestCache,
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

impl Gateway {
    pub(crate) fn new(config: RoutesConfig, metrics: Metrics, cache: RequestCache) -> Result<Self, BoxError> {
        let RoutesConfig {
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
            .collect::<Result<HashMap<_, _>, RequestError>>()?;
        let routes = routes
            .into_iter()
            .map(|(service, route)| Route::new(service.clone(), route, &retry.statuses, &headers.forward, &direct_client, &proxies).map(|route| (service, route)))
            .collect::<Result<HashMap<_, _>, BoxError>>()?;
        for name in proxies.keys() {
            metrics.set_proxy_available(name, false);
        }

        Ok(Self {
            routes,
            proxies,
            cooldowns: RwLock::new(HashMap::new()),
            cooldown: retry.cooldown,
            metrics,
            cache,
        })
    }

    pub(crate) fn start(&self) {
        let mut routes = self.routes.keys().map(String::as_str).collect::<Vec<_>>();
        routes.sort_unstable();
        info_with_fields!(&format!("Routes: {}", routes.join(", ")));
        for (name, proxy) in &self.proxies {
            proxy.start_health_check(name.clone(), self.metrics.clone());
        }
    }

    pub(crate) async fn forward(&self, method: Method, uri: &str, headers: &HeaderMap, body: Vec<u8>) -> Result<ProxyResponse, ProxyError> {
        let route_match = match match_route(&self.routes, &method, uri) {
            Ok(route_match) => route_match,
            Err(error) => {
                let (status, reason, message) = match error {
                    MatchError::NotFound => (Status::NotFound, "route", "route not found"),
                    MatchError::NotAllowed => (Status::Forbidden, "allowlist", "request not allowed"),
                };
                let uri = path::redact(uri);
                AccessLog::rejected(&method, &uri, status.code, reason);
                return Err(ProxyError::new(status, message));
            }
        };
        let route = route_match.route;
        let path = route_match.redacted_path();
        let source = route_match.source;
        let access = AccessLog::new(source, route, &method, &path);
        access.request();
        let _inflight = self.metrics.track_inflight(source, &route.group, &route.service);
        let cache_ttl = self.cache.provider_ttl(&route.group, &route.service, route_match.cache_path(), method.as_str(), &body);
        let cache_key = cache_ttl.map(|_| route_match.cache_key(&method, headers, &body));
        if let Some(key) = &cache_key {
            if let Some(response) = self.cache.get_provider(&route.group, &route.service, key).await {
                self.metrics.record_cache_hit(source, &route.group, &route.service, &path);
                self.metrics.record_response(source, &route.group, &route.service, &path, response.status);
                return Ok(response);
            }
            self.metrics.record_cache_miss(source, &route.group, &route.service, &path);
        }
        let mut candidates = self.available_endpoints(route, &path).await.map_err(|failure| {
            access.unavailable(failure.status, failure.reason);
            self.metrics.record_response(source, &route.group, &route.service, &path, failure.status);
            ProxyError::new(Status::new(failure.status), "no endpoint is available")
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
                access.failover(&failed.name, &failed.host, status);
                self.metrics.record_failover(source, &route.group, &route.service, &failed.name, &path, &reason);
            }
            let host = &endpoint.host;
            let target = match route_match.target_url(endpoint) {
                Ok(target) => target,
                Err(error) => {
                    access.response(&endpoint.name, host, Status::BadRequest.code);
                    self.metrics.record_response(source, &route.group, &route.service, &path, Status::BadRequest.code);
                    return Err(ProxyError::new(Status::BadRequest, error.to_string()));
                }
            };
            if let Some(wait) = endpoint.throttle().await {
                self.metrics.record_throttle_wait(source, &route.group, &route.service, &endpoint.name, wait);
            }
            if !self.endpoint_available(route, endpoint, &path).await {
                continue;
            }
            let started = Instant::now();
            match endpoint.send(&method, target, headers, &route.forward_headers, body.clone()).await {
                Ok((mut response, retry_after)) => {
                    let cacheable = cacheable_response(&response);
                    response.headers = transport::filter_headers(&response.headers, &route.forward_headers);
                    self.metrics
                        .record_upstream_latency(source, &route.group, &route.service, &endpoint.name, response.status, started.elapsed());
                    self.metrics.record_request(source, &route.group, &route.service, &endpoint.name, &path, response.status);
                    if route.should_retry(response.status) {
                        self.start_cooldown(
                            route,
                            endpoint,
                            &path,
                            Failure {
                                status: response.status,
                                reason: "cooldown",
                            },
                            retry_after.unwrap_or(self.cooldown),
                        )
                        .await;
                        let status = response.status;
                        last_response = Some((response, endpoint_index));
                        pending_failover = Some((endpoint_index, status, status.to_string()));
                        continue;
                    }
                    access.response(&endpoint.name, host, response.status);
                    self.metrics.record_response(source, &route.group, &route.service, &path, response.status);
                    if cacheable && let (Some(ttl), Some(key)) = (cache_ttl, cache_key) {
                        self.cache.set_provider(&route.group, &route.service, key, response.clone(), ttl).await;
                    }
                    return Ok(response);
                }
                Err(reason) => {
                    self.metrics
                        .record_request(source, &route.group, &route.service, &endpoint.name, &path, Status::BadGateway.code);
                    self.metrics
                        .record_upstream_latency(source, &route.group, &route.service, &endpoint.name, Status::BadGateway.code, started.elapsed());
                    access.upstream_failed(&endpoint.name, host, reason);
                    self.start_cooldown(
                        route,
                        endpoint,
                        &path,
                        Failure {
                            status: Status::ServiceUnavailable.code,
                            reason,
                        },
                        self.cooldown,
                    )
                    .await;
                    pending_failover = Some((endpoint_index, Status::BadGateway.code, reason.to_string()));
                }
            }
        }

        if let Some((response, endpoint_index)) = last_response {
            let endpoint = &route.endpoints[endpoint_index];
            access.response(&endpoint.name, &endpoint.host, response.status);
            self.metrics.record_response(source, &route.group, &route.service, &path, response.status);
            return Ok(response);
        }
        let failure = self.available_endpoints(route, &path).await.err().unwrap_or(Failure {
            status: Status::ServiceUnavailable.code,
            reason: "upstream",
        });
        access.unavailable(failure.status, failure.reason);
        self.metrics.record_response(source, &route.group, &route.service, &path, failure.status);
        Err(ProxyError::new(Status::new(failure.status), "all upstream requests failed"))
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
                if let Some(cooldown) = cooldowns.get(&endpoint.cooldown_key(&route.group, &route.service, path)) {
                    failures.push(cooldown.failure);
                    return None;
                }
                Some(index)
            })
            .collect::<Vec<_>>();
        if !endpoints.is_empty() {
            return Ok(endpoints);
        }
        let Some(first) = failures.first().copied() else {
            return Err(Failure {
                status: Status::ServiceUnavailable.code,
                reason: "configuration",
            });
        };
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
                .get(&endpoint.cooldown_key(&route.group, &route.service, path))
                .is_none_or(|cooldown| cooldown.until <= Instant::now())
    }

    async fn start_cooldown(&self, route: &Route, endpoint: &Endpoint, path: &str, failure: Failure, duration: Duration) {
        self.metrics.set_cooldown(&route.group, &route.service, &endpoint.name, path, duration);
        self.cooldowns.write().await.insert(
            endpoint.cooldown_key(&route.group, &route.service, path),
            Cooldown {
                until: Instant::now() + duration,
                failure,
            },
        );
    }
}

fn cacheable_response(response: &ProxyResponse) -> bool {
    response.status == 200
        && !response.body.is_empty()
        && !response.headers.contains_key(SET_COOKIE)
        && !response.headers.get_all(CACHE_CONTROL).iter().any(|value| {
            value.to_str().map_or(true, |value| {
                value.split(',').any(|directive| {
                    ["no-store", "no-cache", "private"]
                        .iter()
                        .any(|restricted| directive.trim().split('=').next().is_some_and(|name| name.eq_ignore_ascii_case(restricted)))
                })
            })
        })
        && !response
            .headers
            .get_all(VARY)
            .iter()
            .any(|value| value.to_str().map_or(true, |value| value.split(',').any(|name| name.trim() == "*")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use config::{Config as FileConfig, File, FileFormat};
    use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderName};

    use super::*;
    use crate::config::path::PathAllowlist;
    use crate::config::routes::{EndpointConfig, RouteConfig, Selection};
    use crate::testkit::config::metrics_config;

    #[test]
    fn test_cacheable_response_honors_upstream_directives() {
        let response = ProxyResponse::new(200, HeaderMap::new(), b"body".to_vec());
        assert!(cacheable_response(&response));
        for (header, value) in [
            (CACHE_CONTROL, "max-age=60, no-store"),
            (CACHE_CONTROL, "Private"),
            (CACHE_CONTROL, "no-cache=authorization"),
            (SET_COOKIE, "session=test"),
            (VARY, "Accept, *"),
        ] {
            let mut response = response.clone();
            response.headers.insert(header, value.parse().unwrap());
            assert!(!cacheable_response(&response));
        }
        assert!(!cacheable_response(&ProxyResponse::new(500, HeaderMap::new(), b"body".to_vec())));
        assert!(!cacheable_response(&ProxyResponse::new(200, HeaderMap::new(), Vec::new())));
    }

    #[test]
    fn test_route_header_inheritance() {
        let config = FileConfig::builder()
            .add_source(File::from_str(include_str!("../../testdata/route_headers.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<RoutesConfig>()
            .unwrap();
        let gateway = Gateway::new(config, Metrics::new(metrics_config()), RequestCache::default()).unwrap();
        let routes = &gateway.routes;
        assert_eq!(
            routes["security_tronscan"].forward_headers,
            HashSet::from([ACCEPT, HeaderName::from_static("tron-pro-api-key")])
        );
        assert_eq!(routes["security_goplus"].forward_headers, HashSet::from([ACCEPT, AUTHORIZATION]));
        assert_eq!(routes["security_public"].forward_headers, HashSet::from([ACCEPT]));
    }

    #[test]
    fn test_invalid_route_configuration_rejects_startup() {
        let fixture = include_str!("../../testdata/route_headers.yml");
        for input in [
            fixture.replace("TRON-PRO-API-KEY", "invalid header"),
            fixture.replace("accept", "invalid header"),
            fixture.replace("endpoints: []", "endpoints: [{name: direct, url: 'https://example.invalid', proxy: missing}]"),
        ] {
            let config = FileConfig::builder()
                .add_source(File::from_str(&input, FileFormat::Yaml))
                .build()
                .unwrap()
                .try_deserialize::<RoutesConfig>()
                .unwrap();
            assert!(Gateway::new(config, Metrics::new(metrics_config()), RequestCache::default()).is_err());
        }
    }

    fn gateway() -> Gateway {
        let mut config: RoutesConfig = FileConfig::builder()
            .add_source(File::from_str(include_str!("../../testdata/route_headers.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        config.routes = HashMap::from([(
            "indexer_blockscout".to_string(),
            RouteConfig {
                group: "indexer".to_string(),
                selection: Selection::Ordered,
                headers: None,
                allowlist: PathAllowlist::default(),
                cache: Vec::new(),
                rate: None,
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
            },
        )]);
        Gateway::new(config, Metrics::new(metrics_config()), RequestCache::default()).unwrap()
    }

    #[tokio::test]
    async fn test_cooldowns_preserve_status_and_path() {
        let gateway = gateway();
        let route = gateway.routes.get("indexer_blockscout").unwrap();
        let failure = Failure {
            status: Status::TooManyRequests.code,
            reason: "cooldown",
        };
        for endpoint in &route.endpoints {
            gateway
                .start_cooldown(route, endpoint, "/api/v2/addresses/:value/token-transfers", failure, gateway.cooldown)
                .await;
        }

        let unavailable = gateway.available_endpoints(route, "/api/v2/addresses/:value/token-transfers").await.unwrap_err();
        assert_eq!(unavailable.status, Status::TooManyRequests.code);
        assert_eq!(unavailable.reason, "cooldown");

        let available = gateway.available_endpoints(route, "/api/v2/addresses/:value/token-balances").await.unwrap();
        assert_eq!(available, vec![0, 1]);

        gateway
            .start_cooldown(
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
        gateway.start_cooldown(route, &route.endpoints[1], "/mixed", failure, gateway.cooldown).await;
        let unavailable = gateway.available_endpoints(route, "/mixed").await.unwrap_err();
        assert_eq!(unavailable.status, Status::TooManyRequests.code);
        assert_eq!(unavailable.reason, "cooldown");
    }
}
