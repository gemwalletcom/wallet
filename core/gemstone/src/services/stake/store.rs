use async_trait::async_trait;
use primitives::{AddressName, AssetId, DelegationBase, DelegationValidator, WalletId};

use super::error::GemStakeError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemStakeStore: Send + Sync {
    async fn get_validators(&self, asset_id: AssetId) -> Result<Vec<DelegationValidator>, GemStakeError>;
    async fn save_validators(&self, validators: Vec<DelegationValidator>) -> Result<(), GemStakeError>;
    async fn get_delegation_ids(&self, wallet_id: WalletId, asset_id: AssetId) -> Result<Vec<String>, GemStakeError>;
    async fn update_delegations(&self, wallet_id: WalletId, delegations: Vec<DelegationBase>, delete_ids: Vec<String>) -> Result<(), GemStakeError>;
    async fn save_address_names(&self, names: Vec<AddressName>) -> Result<(), GemStakeError>;
}
