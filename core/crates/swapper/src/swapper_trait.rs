use super::{
    Contracts, SwapperProviderMode, SwapperQuoteData, contract_check,
    cross_chain::VaultAddresses,
    error::SwapperError,
    models::{FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, SwapResult, SwapResultRequest, SwapperChainAsset},
};
use async_trait::async_trait;
use std::fmt::Debug;

use primitives::{AssetId, Chain, swap::SwapStatus};

#[async_trait]
pub trait Swapper: Send + Sync + Debug {
    fn provider(&self) -> &ProviderType;
    fn supported_assets(&self) -> Vec<SwapperChainAsset>;
    fn amount_mode(&self, request: &QuoteRequest) -> SwapAmountMode;
    fn contracts(&self, _from_asset: &AssetId) -> Contracts {
        Contracts::default()
    }
    async fn preload_routes(&self, _from_asset: &AssetId, _to_asset: &AssetId) {}
    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError>;
    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError>;
    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses { deposit: vec![], send: vec![] })
    }
    async fn get_swap_result(&self, _request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        if self.provider().mode == SwapperProviderMode::OnChain {
            Ok(SwapResult::from(SwapStatus::Completed))
        } else {
            Err(SwapperError::NotSupportedAsset)
        }
    }
}

impl<'a> dyn Swapper + 'a {
    pub fn supported_chains(&self) -> Vec<Chain> {
        self.supported_assets().iter().map(SwapperChainAsset::get_chain).collect()
    }

    pub fn verify_contracts(&self, quote: &Quote, data: &SwapperQuoteData) -> Result<(), SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        contract_check::verify(&from_asset, &self.contracts(&from_asset), data)
    }
}
