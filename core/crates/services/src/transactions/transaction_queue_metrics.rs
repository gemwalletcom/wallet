use std::collections::BTreeMap;
use std::time::Duration;

use primitives::{Chain, SwapProvider};
use strum::{AsRefStr, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionQueue {
    Pending,
    InTransit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionQueueGroup {
    pub chain: Chain,
    pub provider: Option<SwapProvider>,
}

impl TransactionQueueGroup {
    pub fn new(chain: Chain, provider: Option<SwapProvider>) -> Self {
        Self { chain, provider }
    }
}

pub trait TransactionQueueMetrics: Send + Sync {
    fn record_queue(&self, queue: TransactionQueue, counts: BTreeMap<TransactionQueueGroup, usize>);
    fn record_completion(&self, queue: TransactionQueue, group: TransactionQueueGroup, to_chain: Option<Chain>, elapsed: Duration);
}
