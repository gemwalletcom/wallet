use chain_traits::{ChainTraits, NodeCheckReporter};
use gem_tracing::{error_fields, info_with_fields};
use primitives::{Chain, NodeCheckProfile, NodeCheckStatus};

struct TracingReporter;

const METHOD_WIDTH: usize = 30;

impl NodeCheckReporter for TracingReporter {
    fn report(&self, method: &str, status: &NodeCheckStatus) {
        match status {
            NodeCheckStatus::Passed { result } => {
                info_with_fields!(&format!("│ ✅     │ {method:<METHOD_WIDTH$} │ {result}"));
            }
            NodeCheckStatus::Failed { error } => {
                error_fields!(&format!("│ ❌     │ {method:<METHOD_WIDTH$} │ {error}"));
            }
        }
    }
}

pub(crate) struct NodeCheckService {
    chain: Chain,
    profile: NodeCheckProfile,
    provider: Box<dyn ChainTraits>,
}

impl NodeCheckService {
    pub(crate) fn new(chain: Chain, profile: NodeCheckProfile, provider: Box<dyn ChainTraits>) -> Self {
        Self { chain, profile, provider }
    }

    pub(crate) async fn run(&self) -> bool {
        let reporter = TracingReporter;
        info_with_fields!(&format!("┌─ {} / {}", self.chain, self.profile.as_ref()));
        info_with_fields!(&format!("│ status │ {:<METHOD_WIDTH$} │ result", "method"));
        let passed = self.provider.check_node(self.profile, &reporter).await.is_healthy();
        if passed {
            info_with_fields!("└─ passed ✅");
        } else {
            error_fields!("└─ failed ❌");
        }
        passed
    }
}
