use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use prometheus_client::registry::Registry;
use services::transactions::{TransactionQueue, TransactionQueueGroup, TransactionQueueMetrics};

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct QueueLabels {
    queue: String,
    chain: String,
    provider: String,
}

impl QueueLabels {
    fn new(queue: TransactionQueue, group: &TransactionQueueGroup) -> Self {
        Self {
            queue: queue.as_ref().to_string(),
            chain: group.chain.as_ref().to_string(),
            provider: group.provider.map(|provider| provider.as_ref().to_string()).unwrap_or_default(),
        }
    }
}

pub struct TransactionMetrics {
    queues: Mutex<BTreeMap<TransactionQueue, BTreeMap<TransactionQueueGroup, usize>>>,
    completion: Family<QueueLabels, Histogram, fn() -> Histogram>,
}

impl Default for TransactionMetrics {
    fn default() -> Self {
        Self {
            queues: Mutex::default(),
            completion: Family::new_with_constructor(|| Histogram::new(exponential_buckets(1.0, 2.0, 18))),
        }
    }
}

impl TransactionQueueMetrics for TransactionMetrics {
    fn record_queue(&self, queue: TransactionQueue, counts: BTreeMap<TransactionQueueGroup, usize>) {
        super::locked(&self.queues).insert(queue, counts);
    }

    fn record_completion(&self, queue: TransactionQueue, group: TransactionQueueGroup, elapsed: Duration) {
        self.completion.get_or_create(&QueueLabels::new(queue, &group)).observe(elapsed.as_secs_f64());
    }
}

impl MetricsProvider for TransactionMetrics {
    fn register(&self, registry: &mut Registry) {
        let queue_size = Family::<QueueLabels, Gauge>::default();
        for (queue, counts) in super::locked(&self.queues).iter() {
            for (group, count) in counts {
                queue_size.get_or_create(&QueueLabels::new(*queue, group)).set(*count as i64);
            }
        }
        registry.register("transactions_queue_size", "Transactions waiting in each queue", queue_size);
        registry.register("transactions_queue_completion_seconds", "Time from entering a queue to its final state", self.completion.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metrics::MetricsRegistry;
    use primitives::{Chain, SwapProvider};

    #[test]
    fn test_register_encodes_queue_metrics() {
        let metrics = TransactionMetrics::default();
        let near_intents = TransactionQueueGroup::new(Chain::Near, Some(SwapProvider::NearIntents));
        metrics.record_queue(TransactionQueue::Pending, BTreeMap::from([(TransactionQueueGroup::new(Chain::HyperCore, None), 2)]));
        metrics.record_queue(TransactionQueue::InTransit, BTreeMap::from([(near_intents, 3)]));
        metrics.record_queue(TransactionQueue::Pending, BTreeMap::from([(TransactionQueueGroup::new(Chain::Solana, None), 1)]));
        metrics.record_completion(TransactionQueue::InTransit, near_intents, Duration::from_secs(90));

        let mut registry = MetricsRegistry::new();
        metrics.register(registry.registry_mut());
        let output = registry.encode();

        assert!(output.contains(r#"transactions_queue_size{queue="pending",chain="solana",provider=""} 1"#));
        assert!(output.contains(r#"transactions_queue_size{queue="in_transit",chain="near",provider="near_intents"} 3"#));
        assert!(!output.contains(r#"chain="hypercore""#));
        assert!(output.contains(r#"transactions_queue_completion_seconds_count{queue="in_transit",chain="near",provider="near_intents"} 1"#));
        assert!(output.contains(r#"transactions_queue_completion_seconds_sum{queue="in_transit",chain="near",provider="near_intents"} 90.0"#));
    }
}
