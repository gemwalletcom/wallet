use std::{collections::HashMap, sync::Arc};

use futures::future;
use primitives::{Chain, NodeCheckRequest};
use settings_chain::node_check_request;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

use super::node_observer::observe_node;
use super::observation::NodeStatusObservation;
use super::selection::{NodeSelectionPolicy, NodeSwitchResult};
use super::switch_reason::NodeSwitchReason;
use super::telemetry::NodeTelemetry;
use crate::config::{ChainConfig, NodeMonitoringConfig, Url};
use crate::metrics::Metrics;
use crate::proxy::NodeDomain;

pub struct NodeMonitor {
    chains: HashMap<Chain, ChainConfig>,
    nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
    metrics: Arc<Metrics>,
    monitoring_config: NodeMonitoringConfig,
}

impl NodeMonitor {
    pub fn new(chains: HashMap<Chain, ChainConfig>, nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>, metrics: Arc<Metrics>, monitoring_config: NodeMonitoringConfig) -> Self {
        Self {
            chains,
            nodes,
            metrics,
            monitoring_config,
        }
    }

    pub fn start_monitoring(&self) {
        for (index, chain_config) in self.chains.values().cloned().enumerate() {
            if let Some(url) = chain_config.urls.first() {
                self.metrics.set_node_host_current(chain_config.chain.as_ref(), &url.host());
            }

            if chain_config.urls.len() <= 1 {
                continue;
            }

            let request = node_check_request(chain_config.chain, self.monitoring_config.profile).unwrap_or(NodeCheckRequest::Basic);

            let nodes = Arc::clone(&self.nodes);
            let metrics = Arc::clone(&self.metrics);
            let interval = chain_config.monitoring_interval(&self.monitoring_config);
            let initial_delay = Duration::from_millis(((index as u64) + 1) * 250);

            tokio::task::spawn(async move {
                sleep(initial_delay).await;

                loop {
                    Self::evaluate_chain(&chain_config, &request, &nodes, &metrics).await;
                    sleep(interval).await;
                }
            });
        }
    }

    async fn evaluate_chain(chain_config: &ChainConfig, request: &NodeCheckRequest, nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, metrics: &Arc<Metrics>) {
        let current_node = match nodes.read().await.get(&chain_config.chain).cloned() {
            Some(node) => node,
            None => {
                NodeTelemetry::log_missing_current(chain_config);
                return;
            }
        };

        let observations = Self::observe_configured_nodes(chain_config, request, &current_node.url).await;
        NodeTelemetry::log_observations(chain_config, &current_node.url, &observations);

        let Some(current_observation) = observations.iter().find(|observation| observation.url == current_node.url) else {
            NodeTelemetry::log_missing_current(chain_config);
            return;
        };
        match NodeSelectionPolicy::select_node(&current_node.url, &observations) {
            Some(switch) => Self::try_switch(chain_config, nodes, metrics, &current_node.url, &switch).await,
            None if current_observation.state.is_healthy() => {}
            None => NodeTelemetry::log_no_candidate(chain_config, &observations),
        }
    }

    async fn observe_configured_nodes(chain_config: &ChainConfig, request: &NodeCheckRequest, current: &Url) -> Vec<NodeStatusObservation> {
        if chain_config.urls.first() == Some(current) {
            let current_observation = observe_node(chain_config.chain, request, current.clone()).await;
            if current_observation.state.is_healthy() {
                return vec![current_observation];
            }

            let fallback_observations = future::join_all(chain_config.urls.iter().skip(1).cloned().map(|url| observe_node(chain_config.chain, request, url))).await;
            return std::iter::once(current_observation).chain(fallback_observations).collect();
        }

        future::join_all(chain_config.urls.iter().cloned().map(|url| observe_node(chain_config.chain, request, url))).await
    }

    async fn try_switch(chain_config: &ChainConfig, nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, metrics: &Arc<Metrics>, current_url: &Url, switch: &NodeSwitchResult<'_>) {
        if Self::switch_node(nodes, metrics, chain_config, current_url, &switch.observation.url, &switch.reason).await {
            NodeTelemetry::log_node_switch(chain_config, current_url, switch)
        }
    }

    async fn switch_node(
        nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>,
        metrics: &Arc<Metrics>,
        chain_config: &ChainConfig,
        expected_current: &Url,
        selected: &Url,
        reason: &NodeSwitchReason,
    ) -> bool {
        let (old_host, new_host) = {
            let mut nodes = nodes.write().await;
            let Some(active_node) = nodes.get(&chain_config.chain) else {
                return false;
            };
            if active_node.url != *expected_current || active_node.url == *selected {
                return false;
            }

            let old_host = active_node.url.host();
            let new_host = selected.host();
            nodes.insert(chain_config.chain, NodeDomain::new(selected.clone(), chain_config.clone()));
            (old_host, new_host)
        };

        metrics.move_node_host_current(chain_config.chain.as_ref(), &old_host, &new_host);
        metrics.add_node_switch(chain_config.chain.as_ref(), &old_host, &new_host, &reason.metric_reason());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;
    use crate::testkit::sync::url;

    #[tokio::test]
    async fn switches_only_when_expected_node_is_current() {
        let chain_config = ChainConfig {
            chain: Chain::Ethereum,
            poll_interval_seconds: None,
            overrides: None,
            allowlist: None,
            cache: None,
            urls: vec![url("https://a"), url("https://b"), url("https://c")],
        };
        let nodes = Arc::new(RwLock::new(HashMap::from([(chain_config.chain, NodeDomain::new(url("https://a"), chain_config.clone()))])));
        let metrics = Arc::new(Metrics::new(MetricsConfig::default()));

        assert!(NodeMonitor::switch_node(&nodes, &metrics, &chain_config, &url("https://a"), &url("https://b"), &NodeSwitchReason::PreferredNode,).await);
        assert_eq!(nodes.read().await.get(&chain_config.chain).unwrap().url, url("https://b"));

        assert!(!NodeMonitor::switch_node(&nodes, &metrics, &chain_config, &url("https://a"), &url("https://c"), &NodeSwitchReason::PreferredNode,).await);
        assert_eq!(nodes.read().await.get(&chain_config.chain).unwrap().url, url("https://b"));
    }
}
