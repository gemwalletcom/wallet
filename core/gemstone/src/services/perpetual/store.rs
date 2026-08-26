use async_trait::async_trait;
use primitives::perpetual::{PerpetualBalance, PerpetualData};
use primitives::{PerpetualPosition, PerpetualProvider};

use super::error::GemPerpetualError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemPerpetualStore: Send + Sync {
    async fn upsert_perpetuals(&self, data: Vec<PerpetualData>) -> Result<(), GemPerpetualError>;
    async fn get_position_ids(&self, wallet_id: String, provider: PerpetualProvider) -> Result<Vec<String>, GemPerpetualError>;
    async fn apply_positions(&self, wallet_id: String, delete_ids: Vec<String>, positions: Vec<PerpetualPosition>) -> Result<(), GemPerpetualError>;
    async fn update_balance(&self, wallet_id: String, balance: PerpetualBalance) -> Result<(), GemPerpetualError>;
}
