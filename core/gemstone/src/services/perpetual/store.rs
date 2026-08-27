use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::perpetual::{PerpetualBalance, PerpetualData};
use primitives::{PerpetualMarketData, PerpetualPosition, PerpetualProvider, WalletId};
use std::collections::HashMap;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPerpetualStore: Send + Sync {
    async fn save_perpetuals(&self, data: Vec<PerpetualData>) -> Result<(), GemServiceError>;
    async fn set_pinned(&self, perpetual_ids: Vec<String>, pinned: bool) -> Result<(), GemServiceError>;
    async fn clear(&self) -> Result<(), GemServiceError>;
    async fn get_positions(&self, wallet_id: WalletId, provider: PerpetualProvider) -> Result<Vec<PerpetualPosition>, GemServiceError>;
    async fn get_position_ids(&self, wallet_id: WalletId, provider: PerpetualProvider) -> Result<Vec<String>, GemServiceError>;
    async fn update_positions(&self, wallet_id: WalletId, positions: Vec<PerpetualPosition>, delete_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn update_balance(&self, wallet_id: WalletId, balance: PerpetualBalance) -> Result<(), GemServiceError>;
    async fn update_market(&self, market: PerpetualMarketData) -> Result<(), GemServiceError>;
    async fn update_prices(&self, prices: HashMap<String, f64>) -> Result<(), GemServiceError>;
}
