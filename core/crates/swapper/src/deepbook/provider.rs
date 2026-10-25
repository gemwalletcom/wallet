use super::{
    client::{Deepbook, PACKAGE_VERSION_DISABLED},
    model::Route,
    tx_builder,
};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData,
    sui_swap::{self, coin_type, discovery_pairs, fee_side, is_move_abort, route_paths, upgraded_error},
};
use async_trait::async_trait;
use futures::future::join_all;
use gem_sui::{ESTIMATION_GAS_BUDGET, tx_builder::PrefetchedTransactionData};
use num_bigint::BigUint;
use primitives::{AssetId, Chain};
use std::collections::HashMap;

#[async_trait]
impl Swapper for Deepbook {
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
        join_all(discovery_pairs(&coin_type(from_asset), &coin_type(to_asset)).iter().map(|(from, to)| self.find_pool(from, to))).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from = coin_type(&request.from_asset.asset_id());
        let to = coin_type(&request.to_asset.asset_id());
        let amount = request.value_u64()?;
        let referral_fee = request.on_chain_referral_fee();
        let fee_side = fee_side(&from, &to);
        let swap_value = referral_fee.swap_value(fee_side, amount);

        let routes = join_all(route_paths(&from, &to).iter().map(|path| self.find_route(path.pairs(&from, &to)))).await;
        let results = join_all(routes.into_iter().flatten().map(|pools| self.quote_pools(pools, &from, &to, swap_value))).await;
        let (routes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        let Some(route) = routes.into_iter().flatten().max_by_key(|route| route.amount_out) else {
            let upgraded = upgraded_error(&self.provider.name);
            let errors = errors.into_iter().filter_map(Result::err).collect::<Vec<_>>();
            return Err(match errors.contains(&upgraded) {
                true => upgraded,
                false => SwapperError::combined(errors).map_min_amount(|minimum| minimum.parse::<u128>().ok().map(|minimum| referral_fee.value_for_swap(fee_side, minimum).to_string())),
            });
        };
        let to_value = referral_fee.transfer(fee_side, amount, route.amount_out).received_value(route.amount_out);

        Quote::from_route(request, self.provider(), BigUint::from(to_value), request.options.slippage.bps, &route, Some(0))
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route: Route = quote.route_data()?;
        let sender = quote.request.wallet_address.as_str();
        let input_coin_type = coin_type(&quote.request.from_asset.asset_id());
        let PrefetchedTransactionData { transaction, input_coins, .. } = PrefetchedTransactionData::prefetch(&self.sui_client, sender, &input_coin_type, None, vec![], &HashMap::new(), ESTIMATION_GAS_BUDGET)
            .await
            .map_err(SwapperError::transaction_error)?;
        let referral_fee = quote.request.on_chain_referral_fee();

        sui_swap::build_quote_data(&self.sui_client, |gas_budget| {
            tx_builder::build_swap_transaction(quote, &route, &referral_fee, &transaction.with_gas_budget(gas_budget), &input_coins)
        })
        .await
        .map_err(|error| match error {
            SwapperError::TransactionError(detail) if is_move_abort(&detail, PACKAGE_VERSION_DISABLED) => upgraded_error(&self.provider.name),
            error => error,
        })
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::super::constants::DEEP_COIN_TYPE;
    use super::*;
    use crate::{SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, cetus_clmm::CetusClmm, models::Options, rank_quotes, testkit::TEST_SUI_WALLET};
    use gem_sui::coin_type_matches;
    use num_bigint::Sign;
    use primitives::{
        asset_constants::{SUI_USDC_TOKEN_ID, SUI_WAL_TOKEN_ID},
        swap::SUI_REFERRAL_ADDRESS,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_deepbook_swaps_simulate() -> Result<(), SwapperError> {
        let provider = Deepbook::new(Arc::new(NativeProvider::default())).unwrap();
        let sui = AssetId::from_chain(Chain::Sui);
        let usdc = AssetId::from_token(Chain::Sui, SUI_USDC_TOKEN_ID);
        let wal = AssetId::from_token(Chain::Sui, SUI_WAL_TOKEN_ID);
        let deep = AssetId::from_token(Chain::Sui, DEEP_COIN_TYPE);

        for (from, to, value, fee_asset) in [
            (&sui, &usdc, 1_071_500_000_000u64, &sui),
            (&usdc, &sui, 200_000_000, &sui),
            (&wal, &usdc, 1_000_000_000_000, &usdc),
            (&deep, &wal, 1_000_000_000, &wal),
        ] {
            let request = QuoteRequest {
                from_asset: SwapperQuoteAsset::from(from.clone()),
                to_asset: SwapperQuoteAsset::from(to.clone()),
                wallet_address: TEST_SUI_WALLET.to_string(),
                destination_address: TEST_SUI_WALLET.to_string(),
                value: BigUint::from(value),
                options: Options::new_with_slippage(50.into()),
            };
            let quote = provider.get_quote(&request).await?;
            let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            let simulated = provider.sui_client.simulate_encoded_transaction(&quote_data.data).await.map_err(SwapperError::transaction_error)?;
            let gem_fee_paid = simulated.balance_changes.iter().any(|change| {
                change.address.as_deref() == Some(SUI_REFERRAL_ADDRESS)
                    && change.coin_type.as_deref().is_some_and(|coin| coin_type_matches(coin, &coin_type(fee_asset)))
                    && change.amount.as_ref().is_some_and(|amount| amount.sign() == Sign::Plus)
            });

            assert_eq!(simulated.effects.and_then(|effects| effects.status).and_then(|status| status.success), Some(true));
            assert!(gem_fee_paid);
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_deep_to_wal_routes_through_a_hub_and_competes_with_cetus() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Sui, DEEP_COIN_TYPE)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Sui, SUI_WAL_TOKEN_ID)),
            wallet_address: TEST_SUI_WALLET.to_string(),
            destination_address: TEST_SUI_WALLET.to_string(),
            value: BigUint::from(1_000_000_000u64),
            options: Options::new_with_slippage(50.into()),
        };
        let deepbook = Deepbook::new(rpc_provider.clone()).unwrap().get_quote(&request).await?;
        let cetus = CetusClmm::new(rpc_provider).unwrap().get_quote(&request).await?;

        assert_eq!(deepbook.route_data::<Route>()?.pools.len(), 2);
        assert_eq!(rank_quotes(&request, vec![deepbook, cetus], &[], |_| false).len(), 2);
        Ok(())
    }
}
