use num_bigint::BigInt;

use crate::{AssetId, Chain, SimulationBalanceChange, SimulationSeverity, SimulationWarning, SimulationWarningApproval, SimulationWarningType};

impl SimulationWarning {
    pub fn mock(warning: SimulationWarningType) -> Self {
        Self::new(SimulationSeverity::Warning, warning, None)
    }
}

impl SimulationWarningApproval {
    pub fn mock(value: Option<BigInt>) -> Self {
        Self {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            value,
        }
    }
}

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
