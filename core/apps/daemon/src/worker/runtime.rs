use job_runner::{JobPlan, JobSchedule, JobStatusReporter, ShutdownReceiver};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerRuntime {
    reporter: Arc<dyn JobStatusReporter>,
    schedule: Arc<dyn JobSchedule>,
}

impl WorkerRuntime {
    pub fn new(reporter: Arc<dyn JobStatusReporter>, schedule: Arc<dyn JobSchedule>) -> Self {
        Self { reporter, schedule }
    }

    pub fn plan(&self, shutdown: ShutdownReceiver) -> JobPlan {
        JobPlan::new(self.reporter.clone(), shutdown, self.schedule.clone())
    }
}
