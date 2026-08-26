use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::perpetual::{PerpetualBalance, PerpetualData};
use primitives::{PerpetualPosition, PerpetualProvider, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemPerpetualStore: Send + Sync {
    async fn save_perpetuals(&self, data: Vec<PerpetualData>) -> Result<(), GemServiceError>;
    async fn get_position_ids(&self, wallet_id: WalletId, provider: PerpetualProvider) -> Result<Vec<String>, GemServiceError>;
    async fn update_positions(&self, wallet_id: WalletId, positions: Vec<PerpetualPosition>, delete_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn update_balance(&self, wallet_id: WalletId, balance: PerpetualBalance) -> Result<(), GemServiceError>;
}
