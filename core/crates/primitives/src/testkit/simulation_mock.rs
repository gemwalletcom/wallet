use std::fmt::Display;

use crate::{AssetId, SimulationBalanceChange};

impl SimulationBalanceChange {
    pub fn mock(asset_id: AssetId, value: impl Display, decimals: i32) -> Self {
        Self {
            asset_id,
            value: value.to_string(),
            decimals,
            name: None,
            symbol: None,
        }
    }
}
