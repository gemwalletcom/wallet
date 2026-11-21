use std::fmt::Debug;

use crate::decimal::{push_f64, push_u64};
use crate::gauge::GaugeValue;
use crate::labels::{LabelEncoder, LabelSet};

pub(crate) const TOTAL_SUFFIX: &str = "_total";

#[derive(Clone, Copy, Debug)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

impl MetricType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }

    pub(crate) fn appends_total_suffix(self) -> bool {
        match self {
            Self::Counter => true,
            Self::Gauge | Self::Histogram => false,
        }
    }
}

pub trait Metric: Clone + Debug + Send + Sync + 'static {
    const TYPE: MetricType;

    fn encode<S: LabelSet>(&self, labels: &S, encoder: &mut MetricEncoder<'_>);
}

pub struct MetricEncoder<'a> {
    out: &'a mut String,
    prefix: Option<&'a str>,
    name: &'static str,
    help: &'static str,
}

impl<'a> MetricEncoder<'a> {
    pub(crate) fn new(out: &'a mut String, prefix: Option<&'a str>, name: &'static str, help: &'static str) -> Self {
        Self { out, prefix, name, help }
    }

    pub(crate) fn descriptor(&mut self, metric_type: MetricType) {
        self.out.push_str("# HELP ");
        self.push_name();
        self.out.push(' ');
        self.out.push_str(self.help);
        self.out.push_str(".\n# TYPE ");
        self.push_name();
        self.out.push(' ');
        self.out.push_str(metric_type.as_str());
        self.out.push('\n');
    }

    pub(crate) fn counter(&mut self, labels: &impl LabelSet, value: u64) {
        self.push_sample(TOTAL_SUFFIX, None, labels);
        push_u64(self.out, value);
        self.out.push('\n');
    }

    pub(crate) fn gauge(&mut self, labels: &impl LabelSet, value: impl GaugeValue) {
        self.push_sample("", None, labels);
        value.encode(self.out);
        self.out.push('\n');
    }

    pub(crate) fn histogram(&mut self, labels: &impl LabelSet, sum: f64, count: u64, buckets: &[(f64, u64)]) {
        self.push_sample("_sum", None, labels);
        push_f64(self.out, sum);
        self.out.push('\n');
        self.push_sample("_count", None, labels);
        push_u64(self.out, count);
        self.out.push('\n');
        let mut cumulative = 0;
        for (bound, bucket) in buckets {
            cumulative += bucket;
            self.push_sample("_bucket", Some(*bound), labels);
            push_u64(self.out, cumulative);
            self.out.push('\n');
        }
    }

    fn push_sample(&mut self, suffix: &str, bound: Option<f64>, labels: &impl LabelSet) {
        self.push_name();
        self.out.push_str(suffix);
        let mut encoder = LabelEncoder::new(self.out);
        if let Some(bound) = bound {
            encoder.add_bucket_bound(bound);
        }
        labels.encode(&mut encoder);
        encoder.finish();
        self.out.push(' ');
    }

    fn push_name(&mut self) {
        if let Some(prefix) = self.prefix {
            self.out.push_str(prefix);
            self.out.push('_');
        }
        self.out.push_str(self.name);
    }
}
