use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Client, Proxy};
use tokio::time::MissedTickBehavior;

use crate::config::{ProxyConfig, ProxyHealthConfig};
use crate::metrics::Metrics;

pub(super) struct OutboundProxy {
    health: ProxyHealthConfig,
    pub(super) client: Client,
    available: Arc<AtomicBool>,
}

impl OutboundProxy {
    pub(super) fn new(config: ProxyConfig, timeout: Duration) -> Result<Self, reqwest::Error> {
        let client = build_client(timeout, Some(&config.url))?;
        Ok(Self {
            health: config.health,
            client,
            available: Arc::new(AtomicBool::new(false)),
        })
    }

    pub(super) fn availability(&self) -> Arc<AtomicBool> {
        self.available.clone()
    }

    pub(super) fn start_health_check(&self, name: String, metrics: Metrics) {
        let health = self.health.clone();
        let client = self.client.clone();
        let available = self.available.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(health.interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                let is_available = match client.get(&health.url).send().await {
                    Ok(response) => response.status().is_success() || response.status().is_redirection(),
                    Err(_) => false,
                };
                available.store(is_available, Ordering::Relaxed);
                metrics.set_proxy_available(&name, is_available);
            }
        });
    }
}

pub(super) fn build_client(timeout: Duration, proxy: Option<&str>) -> Result<Client, reqwest::Error> {
    let mut builder = gem_client::builder().timeout(timeout).redirect(Policy::none());
    if let Some(proxy) = proxy {
        builder = builder.proxy(Proxy::all(proxy)?);
    }
    builder.build()
}
