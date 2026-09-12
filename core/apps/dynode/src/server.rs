use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::Chain;
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::config::Config as RocketConfig;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{Method as RocketMethod, Status};
use rocket::outcome::Outcome as RequestOutcome;
use rocket::response::content::RawText;
use rocket::route::{Handler, Outcome, Route};
use rocket::{Build, Request, Rocket, State};

use crate::BoxError;
use crate::cache::RequestCache;
use crate::config::path::path_without_query;
use crate::config::{ChainConfig, Config};
use crate::gateway::Gateway;
use crate::metrics::Metrics;
use crate::node_service::NodeService;
use crate::proxy::{ProxyRequest, ProxyResponse};
use crate::response::ProxyError;
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
    fn target(&self, uri: &str) -> Result<Target<'_>, ProxyError> {
        if let Some(nodes) = &self.nodes {
            if let Some(chain) = parse_chain(uri) {
                return Ok(Target::Node(nodes, chain));
            }
            if self.gateway.is_none() {
                return Err(ProxyError::new(Status::BadRequest, "Invalid chain"));
            }
        }
        self.gateway
            .as_ref()
            .map(Target::Provider)
            .ok_or_else(|| ProxyError::new(Status::NotFound, "route not found"))
    }
}

pub struct Server {
    address: IpAddr,
    port: u16,
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
                settings
                    .webhook
                    .map(DynodeBroadcastWebhookClient::new)
                    .unwrap_or_else(DynodeBroadcastWebhookClient::disabled),
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
            routes: Routes {
                nodes,
                gateway,
                node_limit,
                route_limit,
            },
            metrics,
        })
    }

    fn rocket(self) -> Rocket<Build> {
        let mut methods = vec![
            RocketMethod::Get,
            RocketMethod::Post,
            RocketMethod::Put,
            RocketMethod::Patch,
            RocketMethod::Delete,
            RocketMethod::Options,
            RocketMethod::Head,
        ];
        let mut server = rocket::custom(RocketConfig::figment().merge(("address", self.address)).merge(("port", self.port))).manage(self.metrics);
        if self.routes.nodes.is_some() {
            server = server.mount("/", rocket::routes![root_endpoint]);
        }
        if self.routes.gateway.is_some() {
            methods.extend([RocketMethod::Trace, RocketMethod::Connect]);
        }
        let routes = methods.into_iter().map(|method| Route::new(method, "/<path..>", ProxyHandler)).collect::<Vec<_>>();
        server.manage(self.routes).mount("/", routes).mount("/", rocket::routes![health_endpoint, metrics_endpoint])
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
        let server = self.rocket().ignite().await?;
        info_with_fields!("Server started", address = &server.config().address.to_string(), port = server.config().port);
        server.launch().await?;
        Ok(())
    }
}

#[derive(Clone)]
struct ProxyHandler;

#[rocket::async_trait]
impl Handler for ProxyHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let routes = match request.guard::<&State<Routes>>().await {
            RequestOutcome::Success(state) => state,
            RequestOutcome::Error((status, ())) | RequestOutcome::Forward(status) => return Outcome::error(status),
        };
        let target = match routes.target(request.uri().path().as_str()) {
            Ok(target) => target,
            Err(error) => return Outcome::from(request, error),
        };
        let limit = match target {
            Target::Node(..) => routes.node_limit,
            Target::Provider(_) => routes.route_limit,
        };
        Outcome::from(request, forward_request(request, data, target, limit).await)
    }
}

async fn forward_request(request: &Request<'_>, data: Data<'_>, target: Target<'_>, limit: usize) -> Result<ProxyResponse, ProxyError> {
    let method = Method::from_bytes(request.method().as_str().as_bytes()).map_err(|_| ProxyError::new(Status::BadRequest, "invalid HTTP method"))?;
    let uri = request.uri().to_string();
    let body = read_request_body(data, limit).await?;
    let headers = request_headers(request)?;
    match target {
        Target::Node(service, chain) => {
            if method == Method::TRACE || method == Method::CONNECT {
                return Err(ProxyError::new(Status::NotFound, "route not found"));
            }
            let proxy_request = ProxyRequest::from_http(method, headers, body, &uri, chain).map_err(|status| ProxyError::new(status, "Failed to build request"))?;
            service.handle_request(&proxy_request).await.map_err(|error| {
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
                ProxyError::new(Status::InternalServerError, "Proxy request failed")
            })
        }
        Target::Provider(gateway) => gateway.forward(method, &uri, &headers, body).await,
    }
}

async fn read_request_body(data: Data<'_>, limit: usize) -> Result<Vec<u8>, ProxyError> {
    let body = data
        .open(limit.bytes())
        .into_bytes()
        .await
        .map_err(|error| ProxyError::new(Status::BadRequest, error.to_string()))?;
    if !body.is_complete() {
        return Err(ProxyError::new(Status::PayloadTooLarge, "request body is too large"));
    }
    Ok(body.into_inner())
}

fn request_headers(request: &Request<'_>) -> Result<HeaderMap, ProxyError> {
    let mut headers = HeaderMap::new();
    for header in request.headers().iter() {
        let name = HeaderName::from_bytes(header.name().as_str().as_bytes()).map_err(|error| ProxyError::new(Status::BadRequest, error.to_string()))?;
        let value = HeaderValue::from_str(header.value()).map_err(|error| ProxyError::new(Status::BadRequest, error.to_string()))?;
        headers.append(name, value);
    }
    Ok(headers)
}

fn parse_chain(path: &str) -> Option<Chain> {
    let chain = path_without_query(path).trim_start_matches('/').split('/').next()?;
    Chain::from_str(chain).ok()
}

#[rocket::get("/health")]
fn health_endpoint() -> Status {
    Status::Ok
}

#[rocket::get("/")]
fn root_endpoint() -> &'static str {
    "ok"
}

#[rocket::get("/metrics")]
fn metrics_endpoint(metrics: &State<Metrics>) -> RawText<String> {
    RawText(metrics.get_metrics())
}

#[cfg(test)]
mod tests;
