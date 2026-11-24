use std::sync::Arc;

use gem_jsonrpc::alien::RpcProvider;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType};

use crate::error::YielderError;
use crate::yo::YoEarnProvider;

pub struct Yielder {
    provider: YoEarnProvider,
}

impl Yielder {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self { provider: YoEarnProvider::new(rpc_provider) }
    }

    pub fn get_providers(&self, asset_id: &AssetId) -> Vec<DelegationValidator> {
        self.provider.get_provider(asset_id).into_iter().collect()
    }

    pub async fn get_positions(&self, address: &str, asset_id: &AssetId) -> Result<Vec<DelegationBase>, YielderError> {
        Ok(self.provider.get_position(address, asset_id).await?.into_iter().collect())
    }

    pub async fn get_balance(&self, chain: Chain, address: &str, token_ids: &[String]) -> Result<Vec<AssetBalance>, YielderError> {
        self.provider.get_balance(chain, address, token_ids).await
    }

    pub async fn get_data(&self, asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError> {
        if !self.provider.get_provider(asset_id).is_some_and(|provider| provider.id == earn_type.provider_id()) {
            return Err(YielderError::NotSupportedAsset);
        }
        self.provider.get_data(asset_id, address, value, earn_type).await
    }
}
