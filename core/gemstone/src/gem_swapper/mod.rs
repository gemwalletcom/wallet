mod error;
pub use error::SwapperError;
mod remote_types;
use remote_types::*;
type Swapper = swapper::swapper::GemSwapper;

use crate::alien::{AlienProvider, AlienRpcProvider, coalescing_provider};
use crate::services::node::GemNodeService;
use primitives::AssetId;
use std::sync::Arc;

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    inner: Swapper,
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    pub fn new(rpc_provider: Arc<dyn AlienProvider>, nodes: Arc<GemNodeService>) -> Self {
        let rpc_provider = coalescing_provider(rpc_provider);
        Self {
            inner: Swapper::new(Arc::new(AlienRpcProvider::new(rpc_provider, nodes))),
        }
    }
}

#[cfg(test)]
impl GemSwapper {
    pub fn mock(swappers: Vec<Box<dyn swapper::Swapper>>) -> Self {
        Self { inner: Swapper::mock(swappers) }
    }
}

impl GemSwapper {
    pub async fn get_quote(&self, request: &SwapperQuoteRequest) -> Result<Vec<SwapperQuote>, SwapperError> {
        self.inner.get_quote(request).await
    }

    pub fn supported_chains_for_from_asset(&self, asset_id: &AssetId) -> SwapperAssetList {
        self.inner.supported_chains_for_from_asset(asset_id)
    }
    pub async fn preload_routes(&self, from_asset: AssetId, to_asset: AssetId) {
        self.inner.preload_routes(&from_asset, &to_asset).await
    }
    pub async fn get_quote_by_provider(&self, provider: &SwapperProvider, request: &SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        self.inner.get_quote_by_provider(provider, request).await
    }
    pub async fn get_quote_data(&self, quote: &SwapperQuote, data: FetchQuoteData) -> Result<GemSwapQuoteData, SwapperError> {
        self.inner.get_quote_data(quote, data).await
    }
}
