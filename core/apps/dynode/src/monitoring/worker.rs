use std::{collections::HashMap, sync::Arc};

use futures::future;
use primitives::{Chain, NodeCheckRequest};
use settings_chain::node_check_request;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

use super::node_observer::observe_node;
use super::sync::{NodeSwitchResult, NodeSyncAnalyzer};
use super::telemetry::NodeTelemetry;
use crate::config::{ChainConfig, NodeMonitoringConfig, Url};
use crate::metrics::Metrics;
use crate::monitoring::NodeService;
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
                NodeService::sync_current_node_metric(&self.metrics, chain_config.chain, url);
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
        let current_node = match NodeService::get_node_domain(nodes, chain_config.chain).await {
            Some(node) => node,
            None => {
                NodeTelemetry::log_missing_current(chain_config);
                return;
            }
        };

        let current_observation = observe_node(chain_config.chain, chain_config.node.clone(), request, current_node.url.clone()).await;
        NodeTelemetry::log_status_debug(chain_config, std::slice::from_ref(&current_observation));

        if current_observation.state.is_healthy() {
            NodeTelemetry::log_node_healthy(chain_config, &current_observation);
            return;
        }

        NodeTelemetry::log_node_unhealthy(chain_config, &current_observation);

        let fallback_urls: Vec<Url> = chain_config.urls.iter().filter(|&url| *url != current_node.url).cloned().collect();
        let fallback_statuses = future::join_all(
            fallback_urls
                .into_iter()
                .map(|url| observe_node(chain_config.chain, chain_config.node.clone(), request, url)),
        )
        .await;
        NodeTelemetry::log_status_debug(chain_config, &fallback_statuses);

        let all_observations = std::iter::once(current_observation).chain(fallback_statuses).collect::<Vec<_>>();

        match NodeSyncAnalyzer::select_best_node(&current_node.url, &all_observations) {
            Some(switch) => Self::try_switch(chain_config, nodes, metrics, &current_node.url, &switch).await,
            None => NodeTelemetry::log_no_candidate(chain_config, &all_observations),
        }
    }

    async fn try_switch(chain_config: &ChainConfig, nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, metrics: &Arc<Metrics>, current_url: &Url, switch: &NodeSwitchResult) {
        let new_url = &switch.observation.url;
        if NodeService::switch_node_if_current(nodes, metrics, chain_config, current_url, new_url, &switch.reason)
            .await
            .is_some()
        {
            NodeTelemetry::log_node_switch(chain_config, current_url, switch);
        }
    }
}
