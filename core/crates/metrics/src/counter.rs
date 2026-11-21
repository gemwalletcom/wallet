use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::encoder::{Metric, MetricEncoder, MetricType};
use crate::labels::LabelSet;

#[derive(Clone, Debug, Default)]
pub struct Counter {
    value: Arc<AtomicU64>,
}

impl Counter {
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }
}

impl Metric for Counter {
    const TYPE: MetricType = MetricType::Counter;

    fn encode<S: LabelSet>(&self, labels: &S, encoder: &mut MetricEncoder<'_>) {
        encoder.counter(labels, self.value.load(Ordering::Relaxed));
    }
}
