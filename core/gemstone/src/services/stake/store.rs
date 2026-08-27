use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AddressName, AssetId, DelegationBase, DelegationValidator, StakeProviderType, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemStakeStore: Send + Sync {
    async fn get_apr(&self, asset_id: AssetId, provider_type: StakeProviderType) -> Result<Option<f64>, GemServiceError>;
    async fn get_validators(&self, asset_id: AssetId, provider_type: StakeProviderType) -> Result<Vec<DelegationValidator>, GemServiceError>;
    async fn save_validators(&self, validators: Vec<DelegationValidator>) -> Result<(), GemServiceError>;
    async fn deactivate_validators(&self, asset_id: AssetId, validator_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn get_delegation_ids(&self, wallet_id: WalletId, asset_id: AssetId, provider_type: StakeProviderType) -> Result<Vec<String>, GemServiceError>;
    async fn update_delegations(&self, wallet_id: WalletId, delegations: Vec<DelegationBase>, delete_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn save_address_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError>;
}
