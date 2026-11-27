use crate::{QuoteDataInput, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData};
use async_trait::async_trait;

/// Providers that no longer quote. They stay registered so lookups by provider still find them.
pub const RETIRED_PROVIDERS: [SwapperProvider; 2] = [SwapperProvider::Panora, SwapperProvider::Mayachain];

#[derive(Debug)]
pub struct RetiredSwapper {
    provider: ProviderType,
}

impl RetiredSwapper {
    pub fn new(id: SwapperProvider) -> Self {
        Self { provider: ProviderType::new(id) }
    }
}

#[async_trait]
impl Swapper for RetiredSwapper {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, _request: &QuoteRequest) -> Result<Quote, SwapperError> {
        Err(SwapperError::NotSupportedChain)
    }

    async fn get_quote_data(&self, _quote: &Quote, _data: QuoteDataInput) -> Result<SwapperQuoteData, SwapperError> {
        Err(SwapperError::NotSupportedChain)
    }
}
