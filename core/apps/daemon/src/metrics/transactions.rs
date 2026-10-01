use std::collections::BTreeMap;
use std::sync::Mutex;

use primitives::Chain;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use services::transactions::{TransactionQueue, TransactionQueueMetrics};

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct QueueLabels {
    queue: String,
    chain: String,
}

#[derive(Default)]
pub struct TransactionMetrics {
    queues: Mutex<BTreeMap<TransactionQueue, BTreeMap<Chain, usize>>>,
}

impl TransactionQueueMetrics for TransactionMetrics {
    fn record_queue(&self, queue: TransactionQueue, counts: BTreeMap<Chain, usize>) {
        super::locked(&self.queues).insert(queue, counts);
    }
}

impl MetricsProvider for TransactionMetrics {
    fn register(&self, registry: &mut Registry) {
        let queue_size = Family::<QueueLabels, Gauge>::default();
        for (queue, counts) in super::locked(&self.queues).iter() {
            for (chain, count) in counts {
                let labels = QueueLabels {
                    queue: queue.as_ref().to_string(),
                    chain: chain.as_ref().to_string(),
                };
                queue_size.get_or_create(&labels).set(*count as i64);
            }
        }
        registry.register("transactions_queue_size", "Transactions waiting in each queue", queue_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metrics::MetricsRegistry;

    #[test]
    fn test_register_encodes_queue_sizes() {
        let metrics = TransactionMetrics::default();
        metrics.record_queue(TransactionQueue::Pending, BTreeMap::from([(Chain::HyperCore, 2)]));
        metrics.record_queue(TransactionQueue::InTransit, BTreeMap::from([(Chain::Near, 3)]));
        metrics.record_queue(TransactionQueue::Pending, BTreeMap::from([(Chain::Solana, 1)]));

        let mut registry = MetricsRegistry::new();
        metrics.register(registry.registry_mut());
        let output = registry.encode();

        assert!(output.contains(r#"transactions_queue_size{queue="pending",chain="solana"} 1"#));
        assert!(output.contains(r#"transactions_queue_size{queue="in_transit",chain="near"} 3"#));
        assert!(!output.contains(r#"chain="hypercore""#));
    }
}
