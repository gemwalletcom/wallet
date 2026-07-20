use chain_traits::ChainTraits;
use gem_tracing::{error_fields, info_with_fields};
use primitives::{NodeCheckRequest, NodeCheckStatus};

const METHOD_WIDTH: usize = 30;

pub(crate) struct NodeCheckService {
    request: NodeCheckRequest,
    provider: Box<dyn ChainTraits>,
}

impl NodeCheckService {
    pub(crate) fn new(request: NodeCheckRequest, provider: Box<dyn ChainTraits>) -> Self {
        Self { request, provider }
    }

    pub(crate) async fn run(&self) -> bool {
        info_with_fields!(&format!("┌─ {} / {}", self.provider.get_chain(), self.request.profile()));
        info_with_fields!(&format!("│ status │ {:<METHOD_WIDTH$} │ result", "method"));
        let passed = match self.provider.get_node_status().await {
            Ok(status) => {
                let report = self.provider.check_node(&self.request, &status).await;
                for (method, status) in &report.checks {
                    match status {
                        NodeCheckStatus::Passed { result } => info_with_fields!(&format!("│ ✅     │ {method:<METHOD_WIDTH$} │ {result}")),
                        NodeCheckStatus::Failed { error } => error_fields!(&format!("│ ❌     │ {method:<METHOD_WIDTH$} │ {error}")),
                    }
                }
                report.is_healthy()
            }
            Err(error) => {
                error_fields!(&format!("│ ❌     │ {:<METHOD_WIDTH$} │ {error}", "node_status"));
                false
            }
        };
        if passed {
            info_with_fields!("└─ passed ✅");
        } else {
            error_fields!("└─ failed ❌");
        }
        passed
    }
}
