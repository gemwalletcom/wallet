mod error;
mod permit2;
pub use error::SwapperError;
pub use permit2::permit2_data_to_eip712_json;
use permit2::*;
mod remote_types;
use remote_types::*;
type Swapper = swapper::swapper::GemSwapper;

use crate::alien::{AlienProvider, AlienProviderWrapper, coalescing_provider};
use primitives::AssetId;
use std::sync::Arc;

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    inner: Swapper,
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let rpc_provider = coalescing_provider(rpc_provider);
        Self {
            inner: Swapper::new(Arc::new(AlienProviderWrapper::new(rpc_provider))),
        }
    }

    pub fn supported_chains_for_from_asset(&self, asset_id: &AssetId) -> SwapperAssetList {
        self.inner.supported_chains_for_from_asset(asset_id)
    }

    pub fn get_providers(&self) -> Vec<SwapperProviderType> {
        self.inner.get_providers()
    }

    pub async fn preload_routes(&self, from_asset: AssetId, to_asset: AssetId) {
        self.inner.preload_routes(&from_asset, &to_asset).await
    }

    pub async fn get_quote(&self, request: &SwapperQuoteRequest) -> Result<Vec<SwapperQuote>, SwapperError> {
        self.inner.get_quote(request).await
    }

    pub async fn get_quote_by_provider(&self, provider: SwapperProvider, request: SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        self.inner.get_quote_by_provider(provider, request).await
    }

    pub async fn get_permit2_for_quote(&self, quote: &SwapperQuote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        self.inner.get_permit2_for_quote(quote).await
    }

    pub async fn get_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<GemSwapQuoteData, SwapperError> {
        self.inner.get_quote_data(quote, data).await
    }
}
