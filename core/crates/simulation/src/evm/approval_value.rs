use gem_evm::{eip712::parse_biguint, transaction::is_unlimited_approval};
use num_bigint::BigUint;
use primitives::SimulationHeader;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ApprovalValue {
    Exact(BigUint),
    Unlimited,
}

impl ApprovalValue {
    pub(crate) fn from_raw(raw_value: &str) -> Option<Self> {
        parse_biguint(raw_value).map(Self::from_value)
    }

    pub(crate) fn from_value(value: BigUint) -> Self {
        if is_unlimited_approval(&value) {
            return Self::Unlimited;
        }
        Self::Exact(value)
    }

    pub(crate) fn display_value(&self) -> String {
        match self {
            Self::Exact(value) => value.to_string(),
            Self::Unlimited => "Unlimited".to_string(),
        }
    }

    pub(crate) fn to_simulation_header(&self, asset_id: primitives::AssetId) -> SimulationHeader {
        match self {
            Self::Exact(value) => SimulationHeader {
                asset_id,
                value: Some(value.clone()),
                is_unlimited: false,
            },
            Self::Unlimited => SimulationHeader { asset_id, value: None, is_unlimited: true },
        }
    }
}
