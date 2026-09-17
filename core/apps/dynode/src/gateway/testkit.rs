use std::collections::HashMap;

use reqwest::Client;

use super::endpoint::Endpoint;
use super::route::Route;
use super::service::Gateway;
use crate::cache::RequestCache;
use crate::config::RoutesConfig;
use crate::config::routes::{EndpointConfig, RouteConfig};
use crate::metrics::Metrics;

impl Route {
    pub(super) fn mock(group: &str, service: &str) -> Self {
        let config = RouteConfig {
            group: group.to_string(),
            ..RouteConfig::mock()
        };
        Self::new(service.to_string(), config, &[429, 503], &[], &Client::new(), &HashMap::new()).unwrap()
    }
}

impl Endpoint {
    pub(super) fn mock(url: &str) -> Self {
        let config = EndpointConfig {
            url: url.to_string(),
            ..EndpointConfig::mock()
        };
        Self::new(config, None, &Client::new(), &HashMap::new()).unwrap()
    }
}

impl Gateway {
    pub(super) fn mock() -> Self {
        let config = RoutesConfig {
            routes: HashMap::from([(
                "indexer_blockscout".to_string(),
                RouteConfig {
                    endpoints: ["key_1", "key_2"]
                        .map(|name| EndpointConfig {
                            name: name.to_string(),
                            ..EndpointConfig::mock()
                        })
                        .into(),
                    ..RouteConfig::mock()
                },
            )]),
            ..RoutesConfig::mock()
        };
        Self::new(config, Metrics::mock(), RequestCache::default()).unwrap()
    }
}
