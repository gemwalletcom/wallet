use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use gem_crypto::compare::constant_time_eq;
use gem_tracing::path;
use rand::seq::SliceRandom;
use reqwest::Client;
use url::Url;

use super::BoxError;
use super::endpoint::Endpoint;
use super::proxy::OutboundProxy;
use crate::config::{CallerConfig, RouteConfig, Selection};

pub(super) struct Route {
    pub(super) group: String,
    pub(super) service: String,
    selection: Selection,
    cursor: AtomicUsize,
    statuses: Vec<u16>,
    pub(super) endpoints: Vec<Endpoint>,
}

pub(super) struct RouteMatch<'a> {
    pub(super) caller: &'a str,
    pub(super) route: &'a Route,
    remainder: &'a str,
    query: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum MatchError {
    Unauthorized,
    Forbidden,
    NotFound,
}

impl Route {
    pub(super) fn new(config: RouteConfig, default_statuses: &[u16], direct_client: &Client, proxies: &HashMap<String, OutboundProxy>) -> Result<Self, BoxError> {
        let statuses = config.retry.map_or_else(|| default_statuses.to_vec(), |retry| retry.statuses);
        let rate = config.rate;
        let endpoints = config
            .endpoints
            .into_iter()
            .map(|endpoint| Endpoint::new(endpoint, rate, direct_client, proxies))
            .collect::<Result<Vec<_>, BoxError>>()?;
        Ok(Self {
            group: config.group,
            service: config.service,
            selection: config.selection,
            cursor: AtomicUsize::new(0),
            statuses,
            endpoints,
        })
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

pub(super) fn match_route<'a>(routes: &'a [Route], callers: &'a HashMap<String, CallerConfig>, uri: &'a str) -> Result<RouteMatch<'a>, MatchError> {
    let (path, query) = uri.split_once('?').map_or((uri, None), |(path, query)| (path, Some(query)));
    let path = path.strip_prefix('/').ok_or(MatchError::NotFound)?;
    let (caller, path) = path.split_once('/').ok_or(MatchError::NotFound)?;
    let caller_config = callers.get(caller).ok_or(MatchError::Unauthorized)?;
    let (key, path) = path.split_once('/').ok_or(MatchError::Unauthorized)?;
    if !constant_time_eq(key.as_bytes(), caller_config.key.as_bytes()) {
        return Err(MatchError::Unauthorized);
    }
    let name_end = path.find('/').unwrap_or(path.len());
    let (name, remainder) = path.split_at(name_end);
    let (group, service) = name.split_once('_').ok_or(MatchError::NotFound)?;
    if !caller_config.groups.contains(group) {
        return Err(MatchError::Forbidden);
    }
    routes
        .iter()
        .find(|route| route.group == group && route.service == service)
        .map(|route| RouteMatch { caller, route, remainder, query })
        .ok_or(MatchError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EndpointConfig;
    use std::collections::HashSet;

    fn route(group: &str, service: &str) -> Route {
        Route {
            group: group.to_string(),
            service: service.to_string(),
            selection: Selection::Ordered,
            cursor: AtomicUsize::new(0),
            statuses: vec![429, 503],
            endpoints: Vec::new(),
        }
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

    fn callers() -> HashMap<String, CallerConfig> {
        HashMap::from([(
            "worker".to_string(),
            CallerConfig {
                key: "secret".to_string(),
                groups: HashSet::from(["prices".to_string(), "indexer".to_string()]),
            },
        )])
    }

    fn match_error(result: Result<RouteMatch<'_>, MatchError>) -> MatchError {
        match result {
            Ok(_) => panic!("expected route matching to fail"),
            Err(error) => error,
        }
    }

    #[test]
    fn test_match_route() {
        let routes = vec![route("prices", "tonapi"), route("prices", "tonapi_rates")];
        let callers = callers();
        let matched = match_route(&routes, &callers, "/worker/secret/prices_tonapi_rates/v2").unwrap();
        assert_eq!(matched.caller, "worker");
        assert_eq!(matched.route.group, "prices");
        assert_eq!(matched.route.service, "tonapi_rates");
        assert_eq!(matched.redacted_path(), "/v2");
        assert_eq!(match_error(match_route(&routes, &callers, "/prices_tonapi_rates/v2")), MatchError::Unauthorized);
        assert_eq!(match_error(match_route(&routes, &callers, "/worker/wrong/prices_tonapi/v2")), MatchError::Unauthorized);
        assert_eq!(match_error(match_route(&routes, &callers, "/worker/secret/nft_opensea/v2")), MatchError::Forbidden);
        assert_eq!(match_error(match_route(&routes, &callers, "/worker/secret/prices_tonapi-other")), MatchError::NotFound);
    }

    #[test]
    fn test_target_url() {
        let route = route("prices", "tonapi");
        let callers = callers();
        let matched = match_route(std::slice::from_ref(&route), &callers, "/worker/secret/prices_tonapi/v2/rates/TON%2FUSD?currency=usd").unwrap();
        assert_eq!(
            matched.target_url(&endpoint("https://tonapi.io/api/", HashMap::new())).unwrap().as_str(),
            "https://tonapi.io/api/v2/rates/TON%2FUSD?currency=usd"
        );
    }

    #[test]
    fn test_target_credentials() {
        let route = route("indexer", "blockscout");
        let callers = callers();
        let matched = match_route(std::slice::from_ref(&route), &callers, "/worker/secret/indexer_blockscout/api?apikey=client&chain=1").unwrap();
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
