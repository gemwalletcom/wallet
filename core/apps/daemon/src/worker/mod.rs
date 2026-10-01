pub mod alerter;
pub mod assets;
pub mod context;
pub mod fiat;
pub mod jobs;
pub mod lists;
pub mod perpetuals;
pub mod plan;
pub mod prices;
pub mod rewards;
pub mod runtime;
pub mod search;
pub mod system;
pub mod transactions;

use std::error::Error;

use job_runner::JobHandle;

use crate::model::WorkerService;
use crate::shutdown::ShutdownReceiver;
use crate::worker::context::WorkerContext;

impl WorkerService {
    pub async fn run_jobs(self, context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
        match self {
            WorkerService::Alerter => alerter::jobs(context, shutdown).await,
            WorkerService::Prices => prices::jobs(context, shutdown).await,
            WorkerService::Fiat => fiat::jobs(context, shutdown).await,
            WorkerService::Assets => assets::jobs(context, shutdown).await,
            WorkerService::System => system::jobs(context, shutdown).await,
            WorkerService::Search => search::jobs(context, shutdown).await,
            WorkerService::Rewards => rewards::jobs(context, shutdown).await,
            WorkerService::Transactions => transactions::jobs(context, shutdown).await,
            WorkerService::Perpetuals => perpetuals::jobs(context, shutdown).await,
            WorkerService::Lists => lists::jobs(context, shutdown).await,
        }
    }
}
