use std::collections::HashMap;

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
    pub(super) fn target_url(&self, base: &Url) -> Result<Url, url::ParseError> {
        let mut path = self.remainder.to_string();
        if let Some(query) = self.query {
            path.push('?');
            path.push_str(query);
        }
        Url::parse(&gem_client::build_request_url(base.as_str(), &path))
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

    fn route(name: &str) -> Route {
        Route {
            name: name.to_string(),
            selection: Selection::Ordered,
            statuses: vec![429, 503],
            endpoints: Vec::new(),
        }
    }

    #[test]
    fn test_match_route() {
        let routes = vec![route("tonapi"), route("tonapi_rates")];
        assert_eq!(match_route(&routes, "/tonapi_rates/v2").unwrap().route.name, "tonapi_rates");
        assert_eq!(match_route(&routes, "/tonapi-other").map(|matched| matched.route.name.as_str()), None);
    }

    #[test]
    fn test_target_url() {
        let route = route("tonapi");
        let matched = match_route(std::slice::from_ref(&route), "/tonapi/v2/rates/TON%2FUSD?currency=usd").unwrap();
        assert_eq!(
            matched.target_url(&Url::parse("https://tonapi.io/api/").unwrap()).unwrap().as_str(),
            "https://tonapi.io/api/v2/rates/TON%2FUSD?currency=usd"
        );
    }

    #[test]
    fn test_should_retry() {
        let route = route("tonapi");
        assert!(route.should_retry(StatusCode::TOO_MANY_REQUESTS));
        assert!(!route.should_retry(StatusCode::BAD_REQUEST));
    }
}
