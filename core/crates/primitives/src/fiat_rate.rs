use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::currency::Currency;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Model)]
#[model(swift = "Sendable")]
pub struct FiatRate {
    pub symbol: Currency,
    pub rate: f64,
}

impl FiatRate {
    pub fn multiplier(&self, base: f64) -> f64 {
        self.rate * base
    }
}
