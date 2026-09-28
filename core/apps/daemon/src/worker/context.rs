use crate::model::WorkerService;
use crate::shutdown::ShutdownReceiver;
use crate::worker::plan::JobPlanBuilder;
use crate::worker::runtime::WorkerRuntime;
use services::ConfigCacher;
use services::Services;

#[derive(Clone)]
pub struct WorkerContext {
    services: Services,
    runtime: WorkerRuntime,
    job_filter: Option<String>,
}

impl WorkerContext {
    pub fn new(services: Services, runtime: WorkerRuntime, job_filter: Option<String>) -> Self {
        Self { services, runtime, job_filter }
    }

    pub fn services(&self) -> Services {
        self.services.clone()
    }

    pub fn plan_builder<'a>(&self, worker: WorkerService, config: &'a ConfigCacher, shutdown_rx: ShutdownReceiver) -> JobPlanBuilder<'a> {
        JobPlanBuilder::with_config(worker, self.runtime.plan(shutdown_rx), config).filter(self.job_filter.clone())
    }
}
