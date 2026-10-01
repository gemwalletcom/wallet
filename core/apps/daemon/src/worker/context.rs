use crate::model::WorkerService;
use crate::shutdown::ShutdownReceiver;
use crate::worker::plan::JobPlanBuilder;
use crate::worker::runtime::WorkerRuntime;
use services::ConfigCacher;
use services::Services;
use services::transactions::TransactionQueueMetrics;
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerContext {
    services: Services,
    runtime: WorkerRuntime,
    job_filter: Option<String>,
    transaction_metrics: Arc<dyn TransactionQueueMetrics>,
}

impl WorkerContext {
    pub fn new(services: Services, runtime: WorkerRuntime, job_filter: Option<String>, transaction_metrics: Arc<dyn TransactionQueueMetrics>) -> Self {
        Self {
            services,
            runtime,
            job_filter,
            transaction_metrics,
        }
    }

    pub fn services(&self) -> Services {
        self.services.clone()
    }

    pub fn transaction_metrics(&self) -> Arc<dyn TransactionQueueMetrics> {
        self.transaction_metrics.clone()
    }

    pub fn plan_builder<'a>(&self, worker: WorkerService, config: &'a ConfigCacher, shutdown: ShutdownReceiver) -> JobPlanBuilder<'a> {
        JobPlanBuilder::with_config(worker, self.runtime.plan(shutdown), config).filter(self.job_filter.clone())
    }
}
