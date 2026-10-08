use bytes::Bytes;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{OriginalUri, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use http::header::CONTENT_TYPE;
use http::{Method, StatusCode, Uri};
use http_body_util::LengthLimitError;
use http_server::{ErrorBody, ServeConfig, catch_panic_layer, serve, shutdown_channel, spawn_signal_handler, timeout_layer};
use primitives::Chain;
use reqwest::header::HeaderMap;

use crate::BoxError;
use crate::cache::RequestCache;
use crate::config::path::path_without_query;
use crate::config::{ChainConfig, Config, ServerConfig};
use crate::gateway::Gateway;
use crate::metrics::Metrics;
use crate::node_service::NodeService;
use crate::proxy::{ProxyRequest, ProxyResponse};
use crate::webhook::DynodeBroadcastWebhookClient;

struct Routes {
    nodes: Option<NodeService>,
    gateway: Option<Gateway>,
    node_limit: usize,
    route_limit: usize,
}

enum Target<'a> {
    Node(&'a NodeService, Chain),
    Provider(&'a Gateway),
}

impl Routes {
    fn target(&self, uri: &str) -> Result<Target<'_>, ErrorBody> {
        if let Some(nodes) = &self.nodes {
            if let Some(chain) = parse_chain(uri) {
                return Ok(Target::Node(nodes, chain));
            }
            if self.gateway.is_none() {
                return Err(ErrorBody::new(StatusCode::BAD_REQUEST, "Invalid chain"));
            }
        }
        self.gateway.as_ref().map(Target::Provider).ok_or_else(|| ErrorBody::new(StatusCode::NOT_FOUND, "route not found"))
    }
}

#[derive(Clone)]
struct AppState {
    routes: Arc<Routes>,
    metrics: Metrics,
}

pub struct Server {
    address: IpAddr,
    port: u16,
    server: ServerConfig,
    routes: Routes,
    metrics: Metrics,
}

impl Server {
    pub fn new(config: Config, chains: HashMap<Chain, ChainConfig>) -> Result<Self, BoxError> {
        let address = config.address.parse()?;
        let port = config.port;
        let metrics = Metrics::new(config.metrics);
        let mut node_limit = 0;
        let nodes = if !chains.is_empty() {
            let settings = config.chains.ok_or("chain configuration is required")?;
            node_limit = settings.request.limit;
            let client = gem_client::builder().timeout(settings.request.timeout).build()?;
            let cache = RequestCache::for_chains(&settings.cache, &settings.chain_types, chains.values());
            Some(NodeService::new(
                chains,
                metrics.clone(),
                client,
                settings.chain_types,
                cache,
                settings.retry,
                settings.headers,
                settings.webhook.map(DynodeBroadcastWebhookClient::new).unwrap_or_else(DynodeBroadcastWebhookClient::disabled),
                settings.monitoring,
            ))
        } else {
            None
        };
        let mut route_limit = 0;
        let gateway = if let Some(settings) = config.routes.filter(|settings| !settings.routes.is_empty()) {
            route_limit = settings.request.limit;
            let cache = RequestCache::for_routes(&settings.cache, &settings.routes);
            Some(Gateway::new(settings, metrics.clone(), cache)?)
        } else {
            None
        };
        if nodes.is_none() && gateway.is_none() {
            return Err("configure chains or provider routes".into());
        }
        Ok(Self {
            address,
            port,
            server: config.server,
            routes: Routes { nodes, gateway, node_limit, route_limit },
            metrics,
        })
    }

    fn router(self) -> Router {
        let http_metrics = self.metrics.http().clone();
        let has_nodes = self.routes.nodes.is_some();
        let state = AppState {
            routes: Arc::new(self.routes),
            metrics: self.metrics,
        };
        let mut router = Router::new().route("/health", any(proxy).get(health)).route("/metrics", any(proxy).get(metrics));
        if has_nodes {
            router = router.route("/", any(proxy).get(root));
        }
        router
            .fallback(proxy)
            .with_state(state)
            .layer(timeout_layer(self.server.request.timeout))
            .layer(catch_panic_layer())
            .layer(http_metrics.layer())
    }

    pub async fn launch(mut self) -> Result<(), BoxError> {
        if let Some(nodes) = &mut self.routes.nodes {
            let mut chains = nodes.chains.keys().map(ToString::to_string).collect::<Vec<_>>();
            chains.sort_unstable();
            info_with_fields!(&format!("Chains: {}", chains.join(", ")));
            nodes.start_monitoring();
        }
        if let Some(gateway) = &self.routes.gateway {
            gateway.start();
        }
        let address = SocketAddr::new(self.address, self.port);
        let config = ServeConfig {
            header_read_timeout: self.server.header.timeout,
            grace: self.server.shutdown.timeout,
        };
        let (sender, shutdown) = shutdown_channel();
        spawn_signal_handler(sender);
        serve(self.router(), address, config, shutdown).await
    }
}

async fn proxy(State(state): State<AppState>, OriginalUri(uri): OriginalUri, request: Request) -> Response {
    match forward_request(&state.routes, &uri, request).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

async fn forward_request(routes: &Routes, uri: &Uri, request: Request) -> Result<ProxyResponse, ErrorBody> {
    let target = routes.target(uri.path())?;
    let limit = match target {
        Target::Node(..) => routes.node_limit,
        Target::Provider(_) => routes.route_limit,
    };
    let method = request.method().clone();
    let headers: HeaderMap = request.headers().clone();
    let uri = uri.path_and_query().map(http::uri::PathAndQuery::as_str).unwrap_or("/").to_string();
    let body = read_request_body(request.into_body(), limit).await?;
    match target {
        Target::Node(service, chain) => {
            if method == Method::TRACE || method == Method::CONNECT {
                return Err(ErrorBody::new(StatusCode::NOT_FOUND, "route not found"));
            }
            let proxy_request = ProxyRequest::from_http(method, headers, body, &uri, chain).map_err(|status| ErrorBody::new(status, "Failed to build request"))?;
            service.proxy_request(&proxy_request).await.map_err(|error| {
                error_with_fields!(
                    "Proxy request failed",
                    error.as_ref(),
                    id = proxy_request.id.as_str(),
                    chain = proxy_request.chain.as_ref(),
                    method = proxy_request.method.as_str(),
                    uri = proxy_request.path.as_str(),
                    user_agent = proxy_request.user_agent.as_str(),
                    latency = DurationMs(proxy_request.elapsed()),
                );
                ErrorBody::new(StatusCode::INTERNAL_SERVER_ERROR, "Proxy request failed")
            })
        }
        Target::Provider(gateway) => gateway.forward(method, &uri, &headers, body).await,
    }
}

async fn read_request_body(body: Body, limit: usize) -> Result<Bytes, ErrorBody> {
    match axum::body::to_bytes(body, limit).await {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            let message = error.to_string();
            if error.into_inner().is::<LengthLimitError>() {
                Err(ErrorBody::new(StatusCode::PAYLOAD_TOO_LARGE, "request body is too large"))
            } else {
                Err(ErrorBody::new(StatusCode::BAD_REQUEST, message))
            }
        }
    }
}

fn parse_chain(path: &str) -> Option<Chain> {
    let chain = path_without_query(path).trim_start_matches('/').split('/').next()?;
    Chain::from_str(chain).ok()
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn root() -> &'static str {
    "ok"
}

async fn metrics(State(state): State<AppState>) -> Response {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], state.metrics.get_metrics()).into_response()
}

#[cfg(test)]
mod tests;
