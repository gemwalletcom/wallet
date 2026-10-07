pub mod consumer;
pub mod job;
pub mod parser;
pub mod transactions;

use std::sync::{Arc, Mutex, MutexGuard};

use axum::response::{IntoResponse, Response};
use http::header::CONTENT_TYPE;
use http_server::HttpMetrics;
use metrics::MetricsRegistry;
use prometheus_client::registry::Registry;

const HTTP_METRICS_PREFIX: &str = "daemon";

pub fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub trait MetricsProvider: Send + Sync {
    fn register(&self, registry: &mut Registry);
}

pub struct Metrics {
    providers: Vec<Arc<dyn MetricsProvider>>,
}

impl Metrics {
    pub fn new(providers: Vec<Arc<dyn MetricsProvider>>) -> Self {
        Self { providers }
    }
}

impl MetricsProvider for Metrics {
    fn register(&self, registry: &mut Registry) {
        for provider in &self.providers {
            provider.register(registry);
        }
    }
}

pub fn encode(provider: &dyn MetricsProvider, http_metrics: &HttpMetrics) -> Response {
    let mut registry = MetricsRegistry::new();
    provider.register(registry.registry_mut());
    http_metrics.register(registry.registry_mut().sub_registry_with_prefix(HTTP_METRICS_PREFIX));
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], registry.encode()).into_response()
}
