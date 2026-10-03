use crate::{FeePriority, FeeRate, GasPriceType};

impl FeeRate {
    pub fn mock(priority: FeePriority, gas_price: u64) -> Self {
        FeeRate::new(priority, GasPriceType::regular(gas_price))
    }
}
