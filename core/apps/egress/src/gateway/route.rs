use std::collections::{HashMap, HashSet};

use gem_tracing::path;
use reqwest::{Client, StatusCode};
use url::Url;

use super::BoxError;
use super::endpoint::Endpoint;
use super::proxy::OutboundProxy;
use crate::config::{RouteConfig, Selection};

pub(super) struct Route {
    pub(super) name: String,
    pub(super) selection: Selection,
    statuses: Vec<u16>,
    pub(super) endpoints: Vec<Endpoint>,
}

pub(super) struct RouteMatch<'a> {
    pub(super) route: &'a Route,
    remainder: &'a str,
    query: Option<&'a str>,
}

impl Route {
    pub(super) fn new(config: RouteConfig, default_statuses: &[u16], direct_client: &Client, proxies: &HashMap<String, OutboundProxy>) -> Result<Self, BoxError> {
        let statuses = config.retry.map_or_else(|| default_statuses.to_vec(), |retry| retry.statuses);
        let endpoints = config
            .endpoints
            .into_iter()
            .map(|endpoint| Endpoint::new(endpoint, direct_client, proxies))
            .collect::<Result<Vec<_>, BoxError>>()?;
        Ok(Self {
            name: config.name,
            selection: config.selection,
            statuses,
            endpoints,
        })
    }

    pub(super) fn should_retry(&self, status: StatusCode) -> bool {
        self.statuses.contains(&status.as_u16())
    }
}

impl RouteMatch<'_> {
    pub(super) fn path(&self) -> String {
        path::redact(self.remainder)
    }

    pub(super) fn target_url(&self, endpoint: &Endpoint) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(&gem_client::build_request_url(endpoint.url.as_str(), self.remainder))?;
        if let Some(query) = self.query {
            url.set_query(Some(query));
        }
        if !endpoint.query.is_empty() {
            let replaced = endpoint.query.keys().map(String::as_str).collect::<HashSet<_>>();
            let mut query = url
                .query_pairs()
                .filter(|(name, _)| !replaced.contains(name.as_ref()))
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            query.extend(endpoint.query.iter().map(|(name, value)| (name.clone(), value.clone())));
            url.set_query(None);
            url.query_pairs_mut().extend_pairs(query);
        }
        Ok(url)
    }
}

pub(super) fn match_route<'a>(routes: &'a [Route], uri: &'a str) -> Option<RouteMatch<'a>> {
    let (path, query) = uri.split_once('?').map_or((uri, None), |(path, query)| (path, Some(query)));
    let path = path.strip_prefix('/')?;
    let name_end = path.find('/').unwrap_or(path.len());
    let (name, remainder) = path.split_at(name_end);
    routes.iter().find(|route| route.name == name).map(|route| RouteMatch { route, remainder, query })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EndpointConfig;

    fn route(name: &str) -> Route {
        Route {
            name: name.to_string(),
            selection: Selection::Ordered,
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
            &Client::new(),
            &HashMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn test_match_route() {
        let routes = vec![route("tonapi"), route("tonapi_rates")];
        let matched = match_route(&routes, "/tonapi_rates/v2").unwrap();
        assert_eq!(matched.route.name, "tonapi_rates");
        assert_eq!(matched.path(), "/v2");
        assert_eq!(match_route(&routes, "/tonapi-other").map(|matched| matched.route.name.as_str()), None);
    }

    #[test]
    fn test_target_url() {
        let route = route("tonapi");
        let matched = match_route(std::slice::from_ref(&route), "/tonapi/v2/rates/TON%2FUSD?currency=usd").unwrap();
        assert_eq!(
            matched.target_url(&endpoint("https://tonapi.io/api/", HashMap::new())).unwrap().as_str(),
            "https://tonapi.io/api/v2/rates/TON%2FUSD?currency=usd"
        );
    }

    #[test]
    fn test_target_credentials() {
        let route = route("blockscout");
        let matched = match_route(std::slice::from_ref(&route), "/blockscout/api?apikey=client&chain=1").unwrap();
        let endpoint = endpoint("https://api.blockscout.com", HashMap::from([("apikey".to_string(), "secret".to_string())]));
        assert_eq!(matched.target_url(&endpoint).unwrap().as_str(), "https://api.blockscout.com/api?chain=1&apikey=secret");
    }

    #[test]
    fn test_should_retry() {
        let route = route("tonapi");
        assert!(route.should_retry(StatusCode::TOO_MANY_REQUESTS));
        assert!(!route.should_retry(StatusCode::BAD_REQUEST));
    }
}
