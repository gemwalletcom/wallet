use std::collections::BTreeMap;

use primitives::Chain;
use strum::AsRefStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionQueue {
    Pending,
    InTransit,
}

pub trait TransactionQueueMetrics: Send + Sync {
    fn record_queue(&self, queue: TransactionQueue, counts: BTreeMap<Chain, usize>);
}
