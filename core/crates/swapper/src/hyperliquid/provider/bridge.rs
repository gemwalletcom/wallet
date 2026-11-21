use super::spot::math::scale_units;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use gem_hypercore::core::{actions::user::send_asset::SendAsset, hypercore::send_asset_typed_data};
use number_formatter::BigNumberFormatter;

use primitives::{
    Chain,
    asset_constants::HYPERCORE_CORE_HYPE_TOKEN_ID,
    contract_constants::HYPERCORE_SYSTEM_ADDRESS,
    known_assets::{HYPERCORE_HYPE, HYPEREVM_HYPE},
    swap::{SwapResult, SwapResultRequest, SwapStatus},
};

use crate::{FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData};

#[derive(Debug)]
pub struct HyperCoreBridge {
    provider: ProviderType,
}

impl HyperCoreBridge {
    pub fn new() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::HyperliquidBridge),
        }
    }

    fn is_bridge_request(request: &QuoteRequest) -> bool {
        let from_id = request.from_asset.asset_id();
        let to_id = request.to_asset.asset_id();
        (from_id == HYPERCORE_HYPE.id && to_id == HYPEREVM_HYPE.id) || (from_id == HYPEREVM_HYPE.id && to_id == HYPERCORE_HYPE.id)
    }
}

impl Default for HyperCoreBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Swapper for HyperCoreBridge {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(Chain::HyperCore, vec![HYPERCORE_HYPE.id.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPEREVM_HYPE.id.clone()]),
        ]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Flexible
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if !Self::is_bridge_request(request) {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let to_value = scale_units(request.value.clone(), request.from_asset.decimals, request.to_asset.decimals)?;

        let quote = Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: 0,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: String::new(),
                }],
            },
            request: request.clone(),
            eta_in_seconds: None,
        };

        Ok(quote)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        match quote.request.from_asset.asset_id().chain {
            Chain::HyperCore => {
                let amount = BigNumberFormatter::plain_value(&quote.request.value, quote.request.from_asset.decimals)?;
                let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

                let send_asset = SendAsset::spot(amount, HYPERCORE_SYSTEM_ADDRESS.to_string(), HYPERCORE_CORE_HYPE_TOKEN_ID.to_string(), nonce);
                let typed_data = send_asset_typed_data(send_asset).map_err(SwapperError::TransactionError)?;

                Ok(SwapperQuoteData::new_contract(HYPERCORE_SYSTEM_ADDRESS.to_string(), quote.request.value.clone(), typed_data, None, None))
            }
            Chain::Hyperliquid => Ok(SwapperQuoteData::new_contract(HYPERCORE_SYSTEM_ADDRESS.to_string(), quote.request.value.clone(), "0x".to_string(), None, None)),
            _ => Err(SwapperError::NotSupportedChain),
        }
    }

    async fn get_swap_result(&self, _request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        Ok(SwapResult::from(SwapStatus::Completed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::mock_quote;
    use num_bigint::BigUint;
    use primitives::{Asset, known_assets::HYPERCORE_SPOT_USDC};

    #[tokio::test]
    async fn test_get_quote_moves_only_hype_between_hyperevm_and_hypercore() {
        let bridge = HyperCoreBridge::new();
        let request = |from: &Asset, to: &Asset| QuoteRequest {
            value: BigUint::from(10u64).pow(18),
            ..mock_quote(from.into(), to.into())
        };

        let to_core = bridge.get_quote(&request(&HYPEREVM_HYPE, &HYPERCORE_HYPE)).await.unwrap();

        assert_eq!(to_core.data.provider.id, SwapperProvider::HyperliquidBridge);
        assert_eq!(to_core.to_value, BigUint::from(10u64).pow(8));
        assert!(bridge.get_quote(&request(&HYPERCORE_HYPE, &HYPEREVM_HYPE)).await.is_ok());
        assert_eq!(
            bridge.get_quote(&request(&HYPEREVM_HYPE, &HYPERCORE_SPOT_USDC)).await.unwrap_err(),
            SwapperError::NoQuoteAvailable,
            "a pair other than HYPE to HYPE is a spot order, not a move"
        );
    }
}
