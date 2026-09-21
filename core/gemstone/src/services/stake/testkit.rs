use async_trait::async_trait;
use primitives::{AssetId, DelegationBase, DelegationValidator, StakeProviderType, WalletId};

use super::store::GemStakeStore;
use crate::services::error::GemServiceError;

pub struct UnusedStakeStore;

#[async_trait]
impl GemStakeStore for UnusedStakeStore {
    async fn get_apr(&self, _: AssetId, _: StakeProviderType) -> Result<Option<f64>, GemServiceError> {
        panic!("unexpected stake read")
    }
    async fn get_validators(&self, _: AssetId, _: StakeProviderType) -> Result<Vec<DelegationValidator>, GemServiceError> {
        panic!("unexpected stake read")
    }
    async fn save_validators(&self, _: Vec<DelegationValidator>) -> Result<(), GemServiceError> {
        panic!("unexpected stake write")
    }
    async fn deactivate_validators(&self, _: AssetId, _: Vec<String>) -> Result<(), GemServiceError> {
        panic!("unexpected stake write")
    }
    async fn get_delegation_ids(&self, _: WalletId, _: AssetId, _: StakeProviderType) -> Result<Vec<String>, GemServiceError> {
        panic!("unexpected stake read")
    }
    async fn update_delegations(&self, _: WalletId, _: Vec<DelegationBase>, _: Vec<String>) -> Result<(), GemServiceError> {
        panic!("unexpected stake write")
    }
}
