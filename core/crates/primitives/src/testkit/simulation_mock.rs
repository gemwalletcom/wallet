use num_bigint::BigInt;

use crate::{AssetId, SimulationBalanceChange};

impl SimulationBalanceChange {
    pub fn mock(asset_id: AssetId, value: BigInt, decimals: i32) -> Self {
        Self {
            asset_id,
            value,
            decimals,
            name: None,
            symbol: None,
        }
    }
}
