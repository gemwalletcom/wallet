use super::{client::CetusClmm, constants::CETUS_CLMM_PUBLISHED_AT, model::PoolRoute, tx_builder};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData,
    sui_swap::{coin_type, fee_side},
};
use async_trait::async_trait;
use gem_sui::coin_type_matches;
use num_bigint::BigUint;
use primitives::{AssetId, Chain};

#[async_trait]
impl Swapper for CetusClmm {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Sui)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        if from_asset.chain != Chain::Sui || to_asset.chain != Chain::Sui {
            return;
        }
        let from = coin_type(from_asset);
        let to = coin_type(to_asset);
        self.preload_pair(&from, &to).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_asset = request.from_asset.asset_id();
        let to_asset = request.to_asset.asset_id();
        let amount = request.value_u64()?;

        let from_coin_type = coin_type(&from_asset);
        let to_coin_type = coin_type(&to_asset);
        let fee_side = fee_side(&from_coin_type, &to_coin_type);

        let referral_fee = request.on_chain_referral_fee();
        let swap_amount = referral_fee.swap_value(fee_side, amount);

        let hops = self.find_route_hops(&from_coin_type, &to_coin_type, swap_amount).await?;
        let gross_amount_out = hops.last().map(|h| h.amount_out).unwrap_or_default();
        let route = PoolRoute {
            hops,
            fee: referral_fee.transfer(fee_side, amount, gross_amount_out),
        };

        Quote::from_route(request, self.provider(), BigUint::from(route.net_amount_out()), request.options.slippage.bps, &route, Some(0))
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route: PoolRoute = quote.route_data()?;

        let request_from = coin_type(&quote.request.from_asset.asset_id());
        let request_to = coin_type(&quote.request.to_asset.asset_id());
        if !coin_type_matches(&request_from, route.input_coin_type()) || !coin_type_matches(&request_to, route.output_coin_type()) {
            return Err(SwapperError::InvalidRoute);
        }

        tx_builder::build_quote_data(&self.sui_client, quote, &route, &quote.request.on_chain_referral_fee(), CETUS_CLMM_PUBLISHED_AT).await
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options, testkit::TEST_SUI_WALLET};
    use primitives::{AssetId, asset_constants::SUI_USDC_TOKEN_ID};
    use std::sync::Arc;

    const BLUE_TOKEN_ID: &str = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";

    fn print_quote(label: &str, quote: &Quote) -> Result<PoolRoute, SwapperError> {
        let route: PoolRoute = quote.route_data()?;
        println!(
            "{label} quote: from_value={}, to_value={}, hops={}, gross_out={}, net_out={}, fee_side={:?}, fee_amount={}",
            quote.from_value,
            quote.to_value,
            route.hops.len(),
            route.gross_amount_out(),
            route.net_amount_out(),
            route.fee.side,
            route.fee.value
        );
        for (index, hop) in route.hops.iter().enumerate() {
            println!("{label} hop {}: amount_in={}, amount_out={}, a2b={}, pool_id={}", index + 1, hop.amount_in, hop.amount_out, hop.a2b, hop.pool_id);
        }
        Ok(route)
    }

    fn print_quote_data(label: &str, quote_data: &SwapperQuoteData) {
        println!("{label} quote_data: to={}, value={}, data_len={}, gas_limit={:?}", quote_data.to, quote_data.value, quote_data.data.len(), quote_data.gas_limit);
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_fetch_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            wallet_address: TEST_SUI_WALLET.to_string(),
            destination_address: TEST_SUI_WALLET.to_string(),
            value: BigUint::from(1500000000u64),
            options: Options::new_with_slippage(50.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        (&provider as &dyn Swapper).verify_contracts(&quote, &quote_data)?;
        print_quote("SUI->USDC", &quote)?;
        print_quote_data("SUI->USDC", &quote_data);

        assert!(quote.to_value > BigUint::ZERO);
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.gas_limit.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_fetch_quote_usdc_to_sui() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            wallet_address: TEST_SUI_WALLET.to_string(),
            destination_address: TEST_SUI_WALLET.to_string(),
            value: BigUint::from(100000u64),
            options: Options::new_with_slippage(50.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        (&provider as &dyn Swapper).verify_contracts(&quote, &quote_data)?;
        print_quote("USDC->SUI", &quote)?;
        print_quote_data("USDC->SUI", &quote_data);

        assert!(quote.to_value > BigUint::ZERO);
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.gas_limit.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_discovers_blue_sui_pool() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(BLUE_TOKEN_ID.to_string()))),
            wallet_address: TEST_SUI_WALLET.to_string(),
            destination_address: TEST_SUI_WALLET.to_string(),
            value: BigUint::from(100000000u64),
            options: Options::default(),
        };

        let quote = provider.get_quote(&request).await?;
        print_quote("SUI->BLUE", &quote)?;
        assert!(quote.to_value > BigUint::ZERO);
        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_routes_usdc_to_blue() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(BLUE_TOKEN_ID.to_string()))),
            wallet_address: TEST_SUI_WALLET.to_string(),
            destination_address: TEST_SUI_WALLET.to_string(),
            value: BigUint::from(100000u64),
            options: Options::default(),
        };

        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);
        let route = print_quote("USDC->BLUE", &quote)?;
        assert!(!route.hops.is_empty() && route.hops.len() <= 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_routes_blue_to_usdc() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(BLUE_TOKEN_ID.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            wallet_address: TEST_SUI_WALLET.to_string(),
            destination_address: TEST_SUI_WALLET.to_string(),
            value: BigUint::from(10000000u64),
            options: Options::default(),
        };

        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);
        let route = print_quote("BLUE->USDC", &quote)?;
        assert!(!route.hops.is_empty() && route.hops.len() <= 2);
        Ok(())
    }
}
