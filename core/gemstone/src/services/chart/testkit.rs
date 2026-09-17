use primitives::ChartDateValue;

use super::GemChart;
use super::rules::base_value;

impl GemChart {
    pub fn mock(values: Vec<ChartDateValue>) -> Self {
        Self {
            base_value: base_value(&values),
            values,
            current: None,
        }
    }
}
