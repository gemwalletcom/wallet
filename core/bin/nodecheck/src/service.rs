use std::{fmt::Display, future::Future};

use gem_tracing::{error_fields, info_with_fields};
use primitives::Chain;

use crate::{
    checker::{NodeCheck, NodeCheckMethod, NodeCheckReporter, NodeCheckResult, NodeCheckStatus},
    fixtures::NodeFixture,
};

struct TracingReporter;

const METHOD_WIDTH: usize = 30;

impl NodeCheckReporter for TracingReporter {
    fn report(&self, method: NodeCheckMethod) {
        match method.status {
            NodeCheckStatus::Passed(result) => {
                info_with_fields!(&format!("│ ✅     │ {:<METHOD_WIDTH$} │ {result}", method.method));
            }
            NodeCheckStatus::Failed(error) => {
                error_fields!(&format!("│ ❌     │ {:<METHOD_WIDTH$} │ {error}", method.method));
            }
        }
    }
}

#[derive(Clone, Copy)]
enum NodeCheckSection {
    LoadBalancer,
    Indexer,
}

impl Display for NodeCheckSection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::LoadBalancer => "load_balancer",
            Self::Indexer => "indexer",
        })
    }
}

pub(crate) struct NodeCheckService {
    chain: Chain,
    checker: Box<dyn NodeCheck>,
}

impl NodeCheckService {
    pub(crate) fn new(chain: Chain, checker: Box<dyn NodeCheck>) -> Self {
        Self { chain, checker }
    }

    pub(crate) async fn run(&self, fixture: NodeFixture) -> bool {
        let reporter = TracingReporter;
        let load_balancer = self.run_section(NodeCheckSection::LoadBalancer, self.checker.check_load_balancer(&reporter)).await;
        let indexer = self.run_section(NodeCheckSection::Indexer, self.checker.check_indexer(fixture, &reporter)).await;

        load_balancer && indexer
    }

    async fn run_section(&self, section: NodeCheckSection, check: impl Future<Output = NodeCheckResult>) -> bool {
        info_with_fields!(&format!("┌─ {} / {section}", self.chain));
        info_with_fields!(&format!("│ status │ {:<METHOD_WIDTH$} │ result", "method"));
        let passed = check.await.is_ok();
        if passed {
            info_with_fields!("└─ passed ✅");
        } else {
            error_fields!("└─ failed ❌");
        }
        passed
    }
}
