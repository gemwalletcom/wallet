use std::{collections::HashMap, sync::Arc, time::Duration};

use primitives::Chain;
use primitives::node_check_request;
use tokio::sync::RwLock;

use super::chain_monitor::ChainMonitor;
use super::evaluator::NodeHealthEvaluator;
use super::request_failure::RequestFailureSignal;
use crate::config::{ChainConfig, NodeMonitoringConfig, Url};
use crate::metrics::Metrics;
use crate::proxy::NodeDomain;

pub(crate) struct NodeMonitor {
    monitors: Vec<ChainMonitor>,
    signals: HashMap<Chain, RequestFailureSignal>,
}

impl NodeMonitor {
    pub(crate) fn new(
        chains: impl IntoIterator<Item = ChainConfig>,
        nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
        metrics: Arc<Metrics>,
        monitoring_config: NodeMonitoringConfig,
    ) -> Self {
        let mut monitors = Vec::new();
        let mut signals = HashMap::new();
        if !monitoring_config.enabled {
            return Self { monitors, signals };
        }

        for (index, chain_config) in chains.into_iter().enumerate() {
            let Some(url) = chain_config.urls.first().cloned() else {
                continue;
            };
            let chain = chain_config.chain;
            metrics.set_node_host_current(chain.as_ref(), &url.host());
            if chain_config.urls.len() <= 1 {
                continue;
            }

            let request = node_check_request(chain, monitoring_config.profile);
            let interval = chain_config.monitoring_interval(&monitoring_config);
            let latency_threshold = chain_config.monitoring_latency(&monitoring_config);
            let initial_delay = Duration::from_millis(((index as u64) + 1) * 250);
            let signal = RequestFailureSignal::new(url, monitoring_config.trigger.clone());
            let evaluator = NodeHealthEvaluator::new(chain_config, latency_threshold, request, Arc::clone(&nodes), Arc::clone(&metrics));
            monitors.push(ChainMonitor::new(evaluator, interval, initial_delay, signal.clone()));
            signals.insert(chain, signal);
        }

        Self { monitors, signals }
    }

    pub(crate) fn start(&mut self) {
        for monitor in self.monitors.drain(..) {
            tokio::spawn(monitor.run());
        }
    }

    pub(crate) fn report(&self, chain: Chain, url: &Url, failed: bool) {
        if let Some(signal) = self.signals.get(&chain) {
            signal.report(url, failed);
        }
    }
}
