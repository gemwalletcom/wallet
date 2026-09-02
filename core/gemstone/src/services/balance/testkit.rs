use num_bigint::BigUint;
use primitives::{AssetId, Chain};

use super::GemAssetBalance;

impl GemAssetBalance {
    pub fn mock() -> Self {
        GemAssetBalance {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            available: BigUint::ZERO,
            frozen: BigUint::ZERO,
            locked: BigUint::ZERO,
            staked: BigUint::ZERO,
            pending: BigUint::ZERO,
            pending_unconfirmed: BigUint::ZERO,
            rewards: BigUint::ZERO,
            reserved: BigUint::ZERO,
            withdrawable: BigUint::ZERO,
            earn: BigUint::ZERO,
            metadata: None,
        }
    }
}
