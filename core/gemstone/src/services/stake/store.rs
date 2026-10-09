use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetId, DelegationValidator, StakeProviderType, WalletId};

use super::model::GemDelegationRecord;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemStakeStore: Send + Sync {
    async fn get_validators(&self, asset_id: AssetId, provider_type: StakeProviderType) -> Result<Vec<DelegationValidator>, GemServiceError>;
    async fn save_validators(&self, validators: Vec<DelegationValidator>) -> Result<(), GemServiceError>;
    async fn deactivate_validators(&self, asset_id: AssetId, validator_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn get_delegation_ids(&self, wallet_id: WalletId, asset_id: AssetId, provider_type: StakeProviderType) -> Result<Vec<String>, GemServiceError>;
    async fn update_delegations(&self, wallet_id: WalletId, delegations: Vec<GemDelegationRecord>, delete_ids: Vec<String>) -> Result<(), GemServiceError>;
}
