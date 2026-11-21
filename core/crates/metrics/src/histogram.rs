use std::iter;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::encoder::{Metric, MetricEncoder, MetricType};
use crate::labels::LabelSet;

#[derive(Clone, Debug)]
pub struct Histogram {
    state: Arc<Mutex<HistogramState>>,
}

#[derive(Clone, Debug)]
struct HistogramState {
    sum: f64,
    count: u64,
    buckets: Vec<(f64, u64)>,
}

impl Histogram {
    pub fn new(bounds: impl IntoIterator<Item = f64>) -> Self {
        let buckets = bounds.into_iter().filter(|bound| bound.is_finite()).chain(iter::once(f64::INFINITY)).map(|bound| (bound, 0)).collect();
        Self {
            state: Arc::new(Mutex::new(HistogramState { sum: 0.0, count: 0, buckets })),
        }
    }

    pub fn observe(&self, value: f64) {
        let mut state = self.lock();
        state.sum += value;
        state.count += 1;
        let bucket = if value.is_nan() { state.buckets.last_mut() } else { state.buckets.iter_mut().find(|(bound, _)| *bound >= value) };
        if let Some((_, count)) = bucket {
            *count += 1;
        }
    }

    fn lock(&self) -> MutexGuard<'_, HistogramState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Metric for Histogram {
    const TYPE: MetricType = MetricType::Histogram;

    fn encode<S: LabelSet>(&self, labels: &S, encoder: &mut MetricEncoder<'_>) {
        let state = self.lock().clone();
        encoder.histogram(labels, state.sum, state.count, &state.buckets);
    }
}

pub fn exponential_buckets(start: f64, factor: f64, length: u16) -> impl Iterator<Item = f64> {
    (0..length).map(move |index| start * factor.powf(f64::from(index)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::LabelPairs;
    use crate::{Family, MetricsRegistry};

    #[test]
    fn test_observe() {
        let mut registry = MetricsRegistry::new();
        let latency = Family::<LabelPairs, Histogram>::new_with_constructor(|| Histogram::new([10.0, 100.0, 1000.0]));
        let bounds = Family::<LabelPairs, Histogram>::new_with_constructor(|| Histogram::new([1.0, f64::INFINITY, f64::NAN, f64::NEG_INFINITY, 2.0]));
        registry.register("latency", "Latency", latency.clone());
        registry.register("bounds", "Bounds", bounds.clone());
        let histogram = latency.get_or_create(&LabelPairs::new(&[]));
        for value in [3.0, 4.0, 15.0, 101.0, 1111.0] {
            histogram.observe(value);
        }
        let histogram = bounds.get_or_create(&LabelPairs::new(&[]));
        for value in [0.5, 1.0, f64::INFINITY, f64::NAN] {
            histogram.observe(value);
        }

        assert_eq!(
            registry.encode(),
            concat!(
                "# HELP latency Latency.\n",
                "# TYPE latency histogram\n",
                "latency_sum 1234.0\n",
                "latency_count 5\n",
                "latency_bucket{le=\"10.0\"} 2\n",
                "latency_bucket{le=\"100.0\"} 3\n",
                "latency_bucket{le=\"1000.0\"} 4\n",
                "latency_bucket{le=\"+Inf\"} 5\n",
                "# HELP bounds Bounds.\n",
                "# TYPE bounds histogram\n",
                "bounds_sum NaN\n",
                "bounds_count 4\n",
                "bounds_bucket{le=\"1.0\"} 2\n",
                "bounds_bucket{le=\"2.0\"} 2\n",
                "bounds_bucket{le=\"+Inf\"} 4\n",
                "# EOF\n",
            )
        );
    }

    #[test]
    fn test_exponential_buckets() {
        assert_eq!(exponential_buckets(1.0, 2.0, 10).collect::<Vec<_>>(), vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0]);
        assert_eq!(exponential_buckets(10.0, 2.0, 0).count(), 0);
    }
}
