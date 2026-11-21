use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use crate::encoder::{Metric, MetricEncoder, MetricType};
use crate::labels::LabelSet;

pub trait GaugeValue: Copy + Debug + Default + Display + Send + Sync + 'static {
    type Atomic: Debug + Default + Send + Sync;

    fn store(atomic: &Self::Atomic, value: Self);
    fn load(atomic: &Self::Atomic) -> Self;
}

#[derive(Debug, Default)]
pub struct Gauge<V: GaugeValue = i64> {
    value: Arc<V::Atomic>,
}

impl<V: GaugeValue> Gauge<V> {
    pub fn set(&self, value: V) {
        V::store(&self.value, value);
    }
}

impl<V: GaugeValue> Clone for Gauge<V> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone() }
    }
}

impl Gauge {
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<V: GaugeValue> Metric for Gauge<V> {
    const TYPE: MetricType = MetricType::Gauge;

    fn encode<S: LabelSet>(&self, labels: &S, encoder: &mut MetricEncoder<'_>) {
        encoder.gauge(labels, V::load(&self.value));
    }
}

impl GaugeValue for i64 {
    type Atomic = AtomicI64;

    fn store(atomic: &AtomicI64, value: i64) {
        atomic.store(value, Ordering::Relaxed);
    }

    fn load(atomic: &AtomicI64) -> i64 {
        atomic.load(Ordering::Relaxed)
    }
}

impl GaugeValue for u64 {
    type Atomic = AtomicU64;

    fn store(atomic: &AtomicU64, value: u64) {
        atomic.store(value, Ordering::Relaxed);
    }

    fn load(atomic: &AtomicU64) -> u64 {
        atomic.load(Ordering::Relaxed)
    }
}
