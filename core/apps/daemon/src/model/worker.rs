use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, EnumIter, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum WorkerService {
    Alerter,
    Prices,
    Fiat,
    Assets,
    System,
    Search,
    Rewards,
    Transactions,
    Perpetuals,
}

impl WorkerService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct WorkerOptions {
    pub service: Option<WorkerService>,
    pub job: Option<String>,
}
