use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use gem_encoding::encode_base64_url;
use gem_hash::sha2::sha256;
use gem_tracing::path;
use rand::seq::SliceRandom;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Client, Method};
use url::Url;

use super::endpoint::Endpoint;
use super::proxy::OutboundProxy;
use crate::BoxError;
use crate::config::path::PathAllowlist;
use crate::config::routes::{RouteConfig, Selection};

pub(super) struct Route {
    pub(super) group: String,
    pub(super) service: String,
    selection: Selection,
    cursor: AtomicUsize,
    statuses: Vec<u16>,
    allowlist: PathAllowlist,
    pub(super) endpoints: Vec<Endpoint>,
    pub(super) forward_headers: HashSet<HeaderName>,
}

pub(super) struct RouteMatch<'a> {
    pub(super) source: &'a str,
    pub(super) route: &'a Route,
    remainder: &'a str,
    query: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum MatchError {
    NotFound,
    NotAllowed,
}

impl Route {
    pub(super) fn new(
        service: String,
        config: RouteConfig,
        default_statuses: &[u16],
        default_headers: &[String],
        direct_client: &Client,
        proxies: &HashMap<String, OutboundProxy>,
    ) -> Result<Self, BoxError> {
        let forward_headers = default_headers
            .iter()
            .chain(config.headers.iter().flat_map(|headers| &headers.forward))
            .map(|header| HeaderName::from_bytes(header.as_bytes()))
            .collect::<Result<HashSet<_>, _>>()?;
        let statuses = config.retry.map_or_else(|| default_statuses.to_vec(), |retry| retry.statuses);
        let rate = config.rate;
        let endpoints = config
            .endpoints
            .into_iter()
            .map(|endpoint| Endpoint::new(endpoint, rate, direct_client, proxies))
            .collect::<Result<Vec<_>, BoxError>>()?;
        Ok(Self {
            group: config.group,
            service,
            selection: config.selection,
            cursor: AtomicUsize::new(0),
            statuses,
            allowlist: config.allowlist,
            endpoints,
            forward_headers,
        })
    }

    pub(super) fn allows(&self, method: &Method, path: &str) -> bool {
        self.allowlist.allows(method.as_str(), path)
    }

    pub(super) fn should_retry(&self, status: u16) -> bool {
        self.statuses.contains(&status)
    }

    pub(super) fn prioritize_endpoints(&self, endpoints: &mut [usize]) {
        match self.selection {
            Selection::Ordered => {}
            Selection::Random => endpoints.shuffle(&mut rand::rng()),
            Selection::RoundRobin => {
                let offset = self.cursor.fetch_add(1, Ordering::Relaxed) % endpoints.len();
                endpoints.rotate_left(offset);
            }
        }
    }
}

impl RouteMatch<'_> {
    pub(super) fn redacted_path(&self) -> String {
        path::redact(self.remainder)
    }

    pub(super) fn cache_path(&self) -> &str {
        self.remainder
    }

    pub(super) fn cache_key(&self, method: &Method, headers: &HeaderMap, body: &[u8]) -> String {
        let mut values = vec![
            self.source.as_bytes(),
            method.as_str().as_bytes(),
            self.remainder.as_bytes(),
            self.query.unwrap_or("").as_bytes(),
            body,
        ];
        let mut headers = headers.iter().filter(|(name, _)| self.route.forward_headers.contains(*name)).collect::<Vec<_>>();
        headers.sort_by_key(|(name, _)| name.as_str());
        for (name, value) in headers {
            values.extend([name.as_str().as_bytes(), value.as_bytes()]);
        }
        let mut bytes = Vec::new();
        for value in values {
            bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
            bytes.extend_from_slice(value);
        }
        encode_base64_url(&sha256(&bytes))
    }

    pub(super) fn target_url(&self, endpoint: &Endpoint) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(&gem_client::build_request_url(endpoint.url.as_str(), self.remainder))?;
        if let Some(query) = self.query {
            url.set_query(Some(query));
        }
        if !endpoint.query.is_empty() {
            let mut query = url
                .query_pairs()
                .filter(|(name, _)| !endpoint.query.contains_key(name.as_ref()))
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            query.extend(endpoint.query.iter().map(|(name, value)| (name.clone(), value.clone())));
            url.set_query(None);
            url.query_pairs_mut().extend_pairs(query);
        }
        Ok(url)
    }
}

pub(super) fn match_route<'a>(routes: &'a HashMap<String, Route>, method: &Method, uri: &'a str) -> Result<RouteMatch<'a>, MatchError> {
    let (path, query) = uri.split_once('?').map_or((uri, None), |(path, query)| (path, Some(query)));
    let path = path.strip_prefix('/').ok_or(MatchError::NotFound)?;
    let (source, path) = path.split_once('/').ok_or(MatchError::NotFound)?;
    if source.is_empty() {
        return Err(MatchError::NotFound);
    }
    let service_end = path.find('/').unwrap_or(path.len());
    let (service, remainder) = path.split_at(service_end);
    let route = routes.get(service).ok_or(MatchError::NotFound)?;
    if !route.allows(method, remainder) {
        return Err(MatchError::NotAllowed);
    }
    Ok(RouteMatch { source, route, remainder, query })
}

#[cfg(test)]
mod tests {
    use reqwest::header::{AUTHORIZATION, HeaderValue};
    use serde_json::json;

    use super::*;
    use crate::config::routes::EndpointConfig;

    fn route(group: &str, service: &str) -> Route {
        Route {
            group: group.to_string(),
            service: service.to_string(),
            selection: Selection::Ordered,
            cursor: AtomicUsize::new(0),
            statuses: vec![429, 503],
            allowlist: PathAllowlist::default(),
            endpoints: Vec::new(),
            forward_headers: HashSet::new(),
        }
    }

    fn routes(routes: Vec<Route>) -> HashMap<String, Route> {
        routes.into_iter().map(|route| (route.service.clone(), route)).collect()
    }

    fn endpoint(url: &str, query: HashMap<String, String>) -> Endpoint {
        Endpoint::new(
            EndpointConfig {
                name: "key_1".to_string(),
                url: url.to_string(),
                headers: None,
                query: Some(query),
                proxy: None,
            },
            None,
            &Client::new(),
            &HashMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn test_cache_key_isolates_request_context() {
        let mut route = route("prices", "provider");
        route.forward_headers.insert(AUTHORIZATION);
        let headers = HeaderMap::from_iter([(AUTHORIZATION, HeaderValue::from_static("Bearer first"))]);
        let request = RouteMatch {
            source: "api",
            route: &route,
            remainder: "/quote",
            query: Some("asset=one"),
        };
        let key = request.cache_key(&Method::POST, &headers, b"first");
        assert_eq!(key, request.cache_key(&Method::POST, &headers, b"first"));
        assert_ne!(key, request.cache_key(&Method::GET, &headers, b"first"));
        assert_ne!(key, request.cache_key(&Method::POST, &headers, b"second"));
        assert_ne!(key, request.cache_key(&Method::POST, &HeaderMap::new(), b"first"));
        for (source, remainder, query) in [
            ("parser", "/quote", Some("asset=one")),
            ("api", "/other", Some("asset=one")),
            ("api", "/quote", Some("asset=two")),
        ] {
            let other = RouteMatch {
                source,
                route: &route,
                remainder,
                query,
            };
            assert_ne!(key, other.cache_key(&Method::POST, &headers, b"first"));
        }
    }

    #[test]
    fn test_match_route() {
        let routes = routes(vec![route("prices", "tonapi"), route("prices", "tonapi_rates")]);
        for source in ["api", "consumer", "parser"] {
            let uri = format!("/{source}/tonapi_rates/v2");
            let matched = match_route(&routes, &Method::GET, &uri).unwrap();
            assert_eq!(matched.source, source);
            assert_eq!(matched.route.group, "prices");
            assert_eq!(matched.route.service, "tonapi_rates");
            assert_eq!(matched.redacted_path(), "/v2");
        }
        for uri in ["/", "//tonapi/v2", "/api", "/api/prices", "/api/nft/opensea/v2", "/api/tonapi-other"] {
            assert_eq!(match_route(&routes, &Method::GET, uri).err(), Some(MatchError::NotFound));
        }
    }

    #[test]
    fn test_match_route_paths() {
        let relay = Route {
            allowlist: serde_json::from_value(json!([
                { "path": "/quote/v2", "method": "POST" },
                { "path": "/chains", "method": "GET" }
            ]))
            .unwrap(),
            ..route("swap", "relay")
        };
        let routes = routes(vec![relay, route("swap", "jupiter")]);
        assert!(match_route(&routes, &Method::POST, "/worker/relay/quote/v2").is_ok());
        assert!(match_route(&routes, &Method::GET, "/worker/relay/chains").is_ok());
        assert_eq!(match_route(&routes, &Method::GET, "/worker/relay/quote/v2").err(), Some(MatchError::NotAllowed));
        assert_eq!(match_route(&routes, &Method::POST, "/worker/relay/execute").err(), Some(MatchError::NotAllowed));
        assert!(match_route(&routes, &Method::POST, "/worker/jupiter/execute").is_ok());
    }

    #[test]
    fn test_target_url() {
        let routes = routes(vec![route("prices", "tonapi")]);
        let matched = match_route(&routes, &Method::GET, "/worker/tonapi/v2/rates/TON%2FUSD?currency=usd").unwrap();
        assert_eq!(
            matched.target_url(&endpoint("https://tonapi.io/api/", HashMap::new())).unwrap().as_str(),
            "https://tonapi.io/api/v2/rates/TON%2FUSD?currency=usd"
        );
    }

    #[test]
    fn test_target_credentials() {
        let routes = routes(vec![route("indexer", "blockscout")]);
        let matched = match_route(&routes, &Method::GET, "/worker/blockscout/api?apikey=client&chain=1").unwrap();
        let endpoint = endpoint("https://api.blockscout.com", HashMap::from([("apikey".to_string(), "secret".to_string())]));
        assert_eq!(matched.target_url(&endpoint).unwrap().as_str(), "https://api.blockscout.com/api?chain=1&apikey=secret");
    }

    #[test]
    fn test_should_retry() {
        let route = route("prices", "tonapi");
        assert!(route.should_retry(429));
        assert!(!route.should_retry(400));
    }

    #[test]
    fn test_prioritize_endpoints() {
        let route = Route {
            selection: Selection::RoundRobin,
            ..route("indexer", "blockscout")
        };
        let orders = (0..4)
            .map(|_| {
                let mut endpoints = vec![0, 1];
                route.prioritize_endpoints(&mut endpoints);
                endpoints
            })
            .collect::<Vec<_>>();

        assert_eq!(orders, vec![vec![0, 1], vec![1, 0], vec![0, 1], vec![1, 0]]);
    }
}
