use std::{sync::Arc, time::Instant};

use chain_traits::ChainTraits;
use primitives::NodeCheckRequest;

use crate::{
    rate_limit,
    result_table::{ResultStatus, ResultTable},
};

pub(crate) struct NodeCheckService {
    request: Arc<NodeCheckRequest>,
    provider: Arc<dyn ChainTraits>,
}

impl NodeCheckService {
    pub(crate) fn new(request: NodeCheckRequest, provider: Box<dyn ChainTraits>) -> Self {
        Self {
            request: Arc::new(request),
            provider: Arc::from(provider),
        }
    }

    pub(crate) async fn run(&self) -> bool {
        let title = format!("{} / {}", self.provider.get_chain(), self.request.profile());
        let table = ResultTable::start(&title, "method", true);
        let status_started = Instant::now();
        let passed = match self.provider.get_node_status().await {
            Ok(status) => {
                let status_latency = status_started.elapsed();
                let report = self.provider.check_node(self.request.as_ref(), &status, status_latency).await;
                for (method, check) in &report.checks {
                    table.row((&check.status).into(), method, Some(check.latency_ms), check.status.message());
                }
                report.is_healthy()
            }
            Err(error) => {
                table.row(
                    ResultStatus::Failed,
                    "node_status",
                    status_started.elapsed().as_millis().try_into().ok(),
                    &error.to_string(),
                );
                false
            }
        };
        table.finish(passed);
        passed
    }

    pub(crate) async fn run_rate_limit(&self, profile_runs_per_second: u32) -> bool {
        rate_limit::run(Arc::clone(&self.request), Arc::clone(&self.provider), profile_runs_per_second).await
    }
}
