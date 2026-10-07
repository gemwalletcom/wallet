use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::extract::MatchedPath;
use http::{Method, Request, Response};
use prometheus_client::encoding::{EncodeLabelSet, LabelSetEncoder};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use prometheus_client::registry::Registry;
use tower::{Layer, Service};

const UNMATCHED_ROUTE: &str = "unmatched";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RequestLabels {
    method: String,
    route: String,
    status: u16,
}

impl EncodeLabelSet for RequestLabels {
    fn encode(&self, encoder: &mut LabelSetEncoder) -> Result<(), fmt::Error> {
        let status = self.status.to_string();
        [("method", self.method.as_str()), ("route", self.route.as_str()), ("status", status.as_str())].encode(encoder)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RouteLabels {
    method: String,
    route: String,
}

impl EncodeLabelSet for RouteLabels {
    fn encode(&self, encoder: &mut LabelSetEncoder) -> Result<(), fmt::Error> {
        [("method", self.method.as_str()), ("route", self.route.as_str())].encode(encoder)
    }
}

#[derive(Clone)]
pub struct HttpMetrics {
    requests: Family<RequestLabels, Counter>,
    duration: Family<RouteLabels, Histogram>,
}

impl Default for HttpMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMetrics {
    pub fn new() -> Self {
        Self {
            requests: Family::default(),
            duration: Family::new_with_constructor(|| Histogram::new(exponential_buckets(0.001, 2.0, 14))),
        }
    }

    pub fn register(&self, registry: &mut Registry) {
        registry.register("http_requests", "HTTP requests by matched route and status", self.requests.clone());
        registry.register("http_request_duration_seconds", "HTTP request duration by matched route", self.duration.clone());
    }

    pub fn layer(&self) -> MetricsLayer {
        MetricsLayer { metrics: self.clone() }
    }

    fn record(&self, method: &Method, route: &str, status: u16, started: Instant) {
        let method = method.as_str().to_string();
        self.requests
            .get_or_create(&RequestLabels {
                method: method.clone(),
                route: route.to_string(),
                status,
            })
            .inc();
        self.duration.get_or_create(&RouteLabels { method, route: route.to_string() }).observe(started.elapsed().as_secs_f64());
    }
}

#[derive(Clone)]
pub struct MetricsLayer {
    metrics: HttpMetrics,
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner, metrics: self.metrics.clone() }
    }
}

#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
    metrics: HttpMetrics,
}

impl<S, B, R> Service<Request<B>> for MetricsService<S>
where
    S: Service<Request<B>, Response = Response<R>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response<R>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let started = Instant::now();
        let method = request.method().clone();
        let route = request.extensions().get::<MatchedPath>().map(|path| path.as_str().to_string()).unwrap_or_else(|| UNMATCHED_ROUTE.to_string());
        let metrics = self.metrics.clone();
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let response = inner.call(request).await?;
            metrics.record(&method, &route, response.status().as_u16(), started);
            Ok(response)
        })
    }
}
