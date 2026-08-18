use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::currency::Currency;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiatRate {
    pub symbol: Currency,
    pub rate: f64,
}

impl FiatRate {
    pub fn multiplier(&self, base: f64) -> f64 {
        self.rate * base
    }
}
