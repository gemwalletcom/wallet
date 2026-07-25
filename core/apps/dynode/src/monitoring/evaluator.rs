use std::{collections::HashMap, iter, sync::Arc};

use primitives::{Chain, NodeCheckRequest};
use tokio::sync::RwLock;

use super::node_observer::observe_node;
use super::observation::NodeStatusObservation;
use super::selection::NodeSelectionPolicy;
use super::switch_reason::NodeSwitchReason;
use super::telemetry::NodeTelemetry;
use crate::config::{ChainConfig, Url};
use crate::metrics::Metrics;
use crate::proxy::NodeDomain;

pub(super) struct NodeHealthEvaluator {
    chain_config: ChainConfig,
    request: NodeCheckRequest,
    nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
    metrics: Arc<Metrics>,
}

impl NodeHealthEvaluator {
    pub(super) fn new(chain_config: ChainConfig, request: NodeCheckRequest, nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>, metrics: Arc<Metrics>) -> Self {
        Self {
            chain_config,
            request,
            nodes,
            metrics,
        }
    }

    pub(super) async fn is_current(&self, url: &Url) -> bool {
        self.current_url().await.as_ref() == Some(url)
    }

    pub(super) async fn check(&self) -> Option<Url> {
        let current_node = match self.nodes.read().await.get(&self.chain_config.chain).cloned() {
            Some(node) => node,
            None => {
                NodeTelemetry::log_missing_current(self.chain_config.chain);
                return None;
            }
        };

        let observations = self.observe_nodes(&current_node.url).await;

        let Some(current_observation) = observations.iter().find(|observation| observation.url == current_node.url) else {
            NodeTelemetry::log_missing_current(self.chain_config.chain);
            return self.current_url().await;
        };
        match NodeSelectionPolicy::select_node(&current_node.url, &observations) {
            Some(switch) => {
                if self.switch_if_current(&current_node.url, &switch.observation.url, &switch.reason).await {
                    NodeTelemetry::log_node_switch(self.chain_config.chain, &current_node.url, &switch);
                }
            }
            None if current_observation.state.is_healthy() => {}
            None => NodeTelemetry::log_no_candidate(self.chain_config.chain),
        }
        self.current_url().await
    }

    async fn observe_nodes(&self, current: &Url) -> Vec<NodeStatusObservation> {
        let current_observation = self.observe_node(current, current.clone()).await;
        let Some(current_index) = self.chain_config.urls.iter().position(|url| url == current) else {
            return vec![current_observation];
        };
        let current_healthy = current_observation.state.is_healthy();
        if current_index == 0 && current_healthy {
            return vec![current_observation];
        }

        let mut observations: Vec<Option<NodeStatusObservation>> = iter::repeat_with(|| None).take(self.chain_config.urls.len()).collect();
        observations[current_index] = Some(current_observation);
        for index in Self::probe_indices(self.chain_config.urls.len(), current_index, current_healthy) {
            let observation = self.observe_node(current, self.chain_config.urls[index].clone()).await;
            let selected = observation.state.is_healthy();
            observations[index] = Some(observation);
            if selected {
                break;
            }
        }

        observations.into_iter().flatten().collect()
    }

    async fn observe_node(&self, current: &Url, url: Url) -> NodeStatusObservation {
        let observation = observe_node(self.chain_config.chain, &self.request, url).await;
        NodeTelemetry::log_observation(self.chain_config.chain, current, &observation);
        observation
    }

    async fn switch_if_current(&self, expected: &Url, selected: &Url, reason: &NodeSwitchReason) -> bool {
        let (old_host, new_host) = {
            let mut nodes = self.nodes.write().await;
            let Some(active_node) = nodes.get(&self.chain_config.chain) else {
                return false;
            };
            if active_node.url != *expected || active_node.url == *selected {
                return false;
            }

            let old_host = active_node.url.host();
            let new_host = selected.host();
            nodes.insert(self.chain_config.chain, NodeDomain::new(selected.clone(), self.chain_config.clone()));
            (old_host, new_host)
        };

        self.metrics.move_node_host_current(self.chain_config.chain.as_ref(), &old_host, &new_host);
        self.metrics
            .add_node_switch(self.chain_config.chain.as_ref(), &old_host, &new_host, &reason.metric_reason());
        true
    }

    async fn current_url(&self) -> Option<Url> {
        self.nodes.read().await.get(&self.chain_config.chain).map(|node| node.url.clone())
    }

    fn probe_indices(node_count: usize, current_index: usize, current_healthy: bool) -> impl Iterator<Item = usize> {
        let search_end = if current_healthy { current_index } else { node_count };
        (0..search_end).filter(move |index| *index != current_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;
    use crate::testkit::sync::url;

    #[test]
    fn test_probe_order() {
        assert_eq!(NodeHealthEvaluator::probe_indices(3, 0, true).collect::<Vec<_>>(), Vec::<usize>::new());
        assert_eq!(NodeHealthEvaluator::probe_indices(3, 0, false).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(NodeHealthEvaluator::probe_indices(3, 1, true).collect::<Vec<_>>(), vec![0]);
        assert_eq!(NodeHealthEvaluator::probe_indices(3, 1, false).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(NodeHealthEvaluator::probe_indices(3, 2, true).collect::<Vec<_>>(), vec![0, 1]);
    }

    #[tokio::test]
    async fn test_switch_if_current() {
        let chain_config = ChainConfig {
            chain: Chain::Ethereum,
            poll_interval_seconds: None,
            overrides: None,
            allowlist: None,
            cache: None,
            urls: vec![url("https://a"), url("https://b"), url("https://c")],
        };
        let nodes = Arc::new(RwLock::new(HashMap::from([(chain_config.chain, NodeDomain::new(url("https://a"), chain_config.clone()))])));
        let evaluator = NodeHealthEvaluator::new(chain_config, NodeCheckRequest::Basic, nodes, Arc::new(Metrics::new(MetricsConfig::default())));

        assert!(evaluator.switch_if_current(&url("https://a"), &url("https://b"), &NodeSwitchReason::PreferredNode).await);
        assert_eq!(evaluator.nodes.read().await.get(&Chain::Ethereum).unwrap().url, url("https://b"));

        assert!(!evaluator.switch_if_current(&url("https://a"), &url("https://c"), &NodeSwitchReason::PreferredNode).await);
        assert_eq!(evaluator.nodes.read().await.get(&Chain::Ethereum).unwrap().url, url("https://b"));
    }
}
