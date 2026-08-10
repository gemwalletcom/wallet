use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use primitives::{Chain, NodeCheckReport, NodeCheckRequest, NodeStatusState};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use settings_chain::{ProviderConfig, ProviderFactory};

use super::observation::NodeStatusObservation;
use super::switch_reason::NodeMonitorError;
use super::telemetry::NodeTelemetry;
use crate::config::Url;

pub(super) async fn observe_node(chain: Chain, request: &NodeCheckRequest, url: Url) -> NodeStatusObservation {
    NodeTelemetry::log_check_started(chain, &url);
    let started_at = Instant::now();
    let observation = |state| NodeStatusObservation::new(url.clone(), state, started_at.elapsed());
    let healthy_observation = |state, latency| NodeStatusObservation::new(url.clone(), state, latency);

    let headers = match request_headers(&url) {
        Ok(headers) => headers,
        Err(error) => return observation(NodeStatusState::error(error)).with_monitor_error(NodeMonitorError::Request),
    };
    let client = match gem_client::builder().default_headers(headers).build() {
        Ok(client) => client,
        Err(error) => return observation(NodeStatusState::error(error.to_string())).with_monitor_error(NodeMonitorError::Request),
    };
    let config = ProviderConfig::new(chain, &url.url);
    let provider = ProviderFactory::new_provider_with_client(config, client);

    let status_started = Instant::now();
    match provider.get_node_status().await {
        Ok(status) => {
            let report = provider.check_node(request, &status, status_started.elapsed()).await;
            match report.error() {
                Some(error) => observation(NodeStatusState::error(error)).with_monitor_error(NodeMonitorError::NodeCheck),
                None => healthy_observation(NodeStatusState::healthy(status), max_check_latency(&report)),
            }
        }
        Err(error) => observation(NodeStatusState::error(error.to_string())).with_monitor_error(NodeMonitorError::from_error(error.as_ref())),
    }
}

fn max_check_latency(report: &NodeCheckReport) -> Duration {
    report.checks.iter().map(|check| Duration::from_millis(check.latency_ms)).max().unwrap_or_default()
}

fn request_headers(url: &Url) -> Result<HeaderMap, String> {
    match &url.headers {
        Some(headers) => headers
            .iter()
            .map(|(name, value)| {
                Ok((
                    HeaderName::from_str(name).map_err(|error| error.to_string())?,
                    HeaderValue::from_str(value).map_err(|error| error.to_string())?,
                ))
            })
            .collect(),
        None => Ok(HeaderMap::new()),
    }
}
