use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, PoisonError, RwLock};

use crate::encoder::{Metric, MetricEncoder};
use crate::labels::LabelSet;

#[derive(Clone, Debug)]
pub struct Family<S, M> {
    metrics: Arc<RwLock<HashMap<S, M>>>,
    constructor: fn() -> M,
}

pub(crate) trait EncodeFamily: Debug + Send + Sync {
    fn encode(&self, encoder: &mut MetricEncoder<'_>);
}

impl<S: LabelSet, M: Metric> Family<S, M> {
    pub fn new_with_constructor(constructor: fn() -> M) -> Self {
        Self { metrics: Arc::default(), constructor }
    }

    pub fn get_or_create(&self, labels: &S) -> M {
        if let Some(metric) = self.metrics.read().unwrap_or_else(PoisonError::into_inner).get(labels) {
            return metric.clone();
        }
        self.metrics.write().unwrap_or_else(PoisonError::into_inner).entry(labels.clone()).or_insert_with(self.constructor).clone()
    }
}

impl<S: LabelSet, M: Metric + Default> Default for Family<S, M> {
    fn default() -> Self {
        Self::new_with_constructor(M::default)
    }
}

impl<S: LabelSet, M: Metric> EncodeFamily for Family<S, M> {
    fn encode(&self, encoder: &mut MetricEncoder<'_>) {
        let metrics = self.metrics.read().unwrap_or_else(PoisonError::into_inner);
        if metrics.is_empty() {
            return;
        }
        encoder.descriptor(M::TYPE);
        for (labels, metric) in metrics.iter() {
            metric.encode(labels, encoder);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::LabelPairs;
    use crate::{Counter, Histogram, MetricsRegistry};

    #[test]
    fn test_get_or_create() {
        let family = Family::<LabelPairs, Counter>::default();
        let mut registry = MetricsRegistry::new();
        registry.register("requests", "Requests", family.clone());
        let get = LabelPairs::new(&[("method", "GET")]);
        let post = LabelPairs::new(&[("method", "POST")]);
        let held = family.get_or_create(&get);
        family.get_or_create(&post).inc();
        held.inc();
        family.get_or_create(&get).inc();
        family.get_or_create(&post).inc();
        let mut lines = registry.encode().lines().map(str::to_owned).collect::<Vec<_>>();
        lines[2..4].sort();
        assert_eq!(
            lines,
            vec!["# HELP requests Requests.", "# TYPE requests counter", "requests_total{method=\"GET\"} 2", "requests_total{method=\"POST\"} 2", "# EOF",]
        );

        let family = Family::<LabelPairs, Histogram>::new_with_constructor(|| Histogram::new([5.0]));
        let mut registry = MetricsRegistry::new();
        registry.register("latency", "Latency", family.clone());
        family.get_or_create(&LabelPairs::new(&[])).observe(7.0);
        assert_eq!(
            registry.encode(),
            "# HELP latency Latency.\n# TYPE latency histogram\nlatency_sum 7.0\nlatency_count 1\nlatency_bucket{le=\"5.0\"} 0\nlatency_bucket{le=\"+Inf\"} 1\n# EOF\n"
        );
    }
}
