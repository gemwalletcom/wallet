use super::{client::Hyperion, model::token_metadata, tx_builder};
use crate::{Binding, QuoteDataInput, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData, fees::FeeSide};
use async_trait::async_trait;
use gem_client::Client;
use num_bigint::BigUint;
use primitives::{AssetId, Chain};

#[async_trait]
impl<C: Client + 'static> Swapper for Hyperion<C> {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Aptos)]
    }

    fn binding(&self, _quote: &Quote) -> Binding {
        Binding::BuiltOnDevice
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        if let (Ok(from), Ok(to)) = (token_metadata(from_asset), token_metadata(to_asset)) {
            self.preload_pair(&from.to_string(), &to.to_string()).await;
        }
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let amount_in = request.value_u64()?;
        let from = token_metadata(&request.from_asset.asset_id())?.to_string();
        let to = token_metadata(&request.to_asset.asset_id())?.to_string();
        let swap_amount = request.on_chain_referral_fee().swap_value(FeeSide::Input, amount_in);
        let (route, amount_out) = self.find_route(&from, &to, swap_amount).await?;

        Quote::from_route(request, self.provider(), BigUint::from(amount_out), request.options.slippage.bps, &route, Some(0))
    }

    async fn get_quote_data(&self, quote: &Quote, _data: QuoteDataInput) -> Result<SwapperQuoteData, SwapperError> {
        tx_builder::build_quote_data(&self.client, quote, &quote.route_data()?, &quote.request.on_chain_referral_fee()).await
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperQuoteAsset, hyperion::model::PoolRoute, models::Options};
    use gem_aptos::{HYPERION_PARTNER_SWAP_FUNCTION, TransactionPayload};
    use gem_reqwest::NativeProvider;
    use primitives::asset_constants::{APTOS_USDC_ASSET_ID, APTOS_USDT_ASSET_ID};
    use std::sync::Arc;

    const TEST_WALLET: &str = "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef";

    #[tokio::test]
    async fn test_hyperion_get_quote_and_quote_data() -> Result<(), SwapperError> {
        let provider = Hyperion::new(Arc::new(NativeProvider::default())).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Aptos)),
            to_asset: SwapperQuoteAsset::from(APTOS_USDC_ASSET_ID.clone()),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: BigUint::from(100_000_000u64),
            options: Options::default(),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, QuoteDataInput::None).await?;
        (&provider as &dyn Swapper).verify_binding(&quote, &quote_data)?;
        let payload: TransactionPayload = serde_json::from_str(&quote_data.data)?;
        println!("APT->USDC quote: to_value={}, route={}, gas_limit={:?}", quote.to_value, quote.data.routes[0].route_data, quote_data.gas_limit);

        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(payload.function.as_deref(), Some(HYPERION_PARTNER_SWAP_FUNCTION));
        assert!(quote_data.gas_limit.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_hyperion_get_quote_usdt_to_usdc() -> Result<(), SwapperError> {
        let provider = Hyperion::new(Arc::new(NativeProvider::default())).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(APTOS_USDT_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(APTOS_USDC_ASSET_ID.clone()),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: BigUint::from(1_000_000u64),
            options: Options::default(),
        };

        let quote = provider.get_quote(&request).await?;
        let route: PoolRoute = quote.route_data()?;
        println!("USDT->USDC quote: to_value={}, pools={:?}", quote.to_value, route.pools);

        assert!(quote.to_value > BigUint::ZERO);
        assert!(!route.pools.is_empty() && route.pools.len() <= 2);

        Ok(())
    }
}
