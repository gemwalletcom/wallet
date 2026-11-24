use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::extract::MatchedPath;
use http::{Method, Request, Response};
use metrics::{Counter, Family, Histogram, LabelEncoder, LabelSet, MetricsRegistry, exponential_buckets};
use tower::{Layer, Service};

const UNMATCHED_ROUTE: &str = "unmatched";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RequestLabels {
    method: &'static str,
    route: String,
    status: u16,
}

impl LabelSet for RequestLabels {
    fn encode(&self, encoder: &mut LabelEncoder<'_>) {
        encoder.add("method", self.method);
        encoder.add("route", &self.route);
        encoder.add("status", &self.status);
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RouteLabels {
    method: &'static str,
    route: String,
}

impl LabelSet for RouteLabels {
    fn encode(&self, encoder: &mut LabelEncoder<'_>) {
        encoder.add("method", self.method);
        encoder.add("route", &self.route);
    }
}

#[derive(Clone, Debug)]
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

    pub fn register(&self, registry: &mut MetricsRegistry) {
        registry.register("http_requests", "HTTP requests by matched route and status", self.requests.clone());
        registry.register("http_request_duration_seconds", "HTTP request duration by matched route", self.duration.clone());
    }

    pub fn layer(&self) -> MetricsLayer {
        MetricsLayer { metrics: self.clone() }
    }

    fn record(&self, method: &Method, route: &str, status: u16, started: Instant) {
        let method = method_label(method);
        self.requests.get_or_create(&RequestLabels { method, route: route.to_string(), status }).inc();
        self.duration.get_or_create(&RouteLabels { method, route: route.to_string() }).observe(started.elapsed().as_secs_f64());
    }
}

pub fn method_label(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::HEAD => "HEAD",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::DELETE => "DELETE",
        Method::PATCH => "PATCH",
        Method::OPTIONS => "OPTIONS",
        Method::CONNECT => "CONNECT",
        Method::TRACE => "TRACE",
        _ => "OTHER",
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

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::routing::get;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_unknown_methods_share_one_label() {
        let metrics = HttpMetrics::new();
        let mut registry = MetricsRegistry::new();
        metrics.register(&mut registry);
        let router = Router::new().route("/", get(|| async {})).layer(metrics.layer());

        for method in ["GET", "FOO", "BAR"] {
            let request = Request::builder().method(method).uri("/").body(Body::empty()).unwrap();
            router.clone().oneshot(request).await.unwrap();
        }

        let output = registry.encode();
        assert!(output.contains(r#"method="GET""#));
        assert!(output.contains(r#"http_requests_total{method="OTHER",route="/",status="405"} 2"#));
        assert!(!output.contains("FOO") && !output.contains("BAR"));
    }
}
