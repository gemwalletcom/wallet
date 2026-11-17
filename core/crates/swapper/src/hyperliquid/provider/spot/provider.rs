use std::sync::Arc;

use async_trait::async_trait;
use gem_hypercore::{
    core::actions::agent::order::{Builder, PlaceOrder, make_market_order},
    models::{spot::SpotMeta, token::SpotToken},
    perpetual_formatter::MIN_ORDER_VALUE_USD,
    provider::fee_calculator::BUILDER_FEE_RATE_SCALE,
    rpc::client::HyperCoreClient,
};
use num_bigint::BigUint;
use number_formatter::{BigNumberFormatter, Decimal, NumberFormatterError};
use primitives::{
    Chain, HOUR,
    asset_constants::{HYPERCORE_SPOT_HYPE_ASSET_ID, HYPERCORE_SPOT_UBTC_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID},
    known_assets::{HYPERCORE_HYPE, HYPERCORE_SPOT_HYPE, HYPERCORE_SPOT_UBTC, HYPERCORE_SPOT_USDC},
};

use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    client_factory::create_rpc_client,
    error::INVALID_AMOUNT,
    route_cache::Cache,
};

use super::{
    math::{SpotSide, format_decimal, format_order_size, limit_price_with_slippage, round_size_down, scale_units, spot_asset_index},
    simulator::{simulate_buy, simulate_sell},
};

const HYPE_USDC_MARKET_INDEX: u32 = 107;
const UBTC_USDC_MARKET_INDEX: u32 = 142;

fn shipped_market_index(base_token: &SpotToken, quote_token: &SpotToken) -> Option<u32> {
    let (base, quote) = (base_token.asset_id(Chain::HyperCore), quote_token.asset_id(Chain::HyperCore));
    if quote != *HYPERCORE_SPOT_USDC_ASSET_ID {
        return None;
    }
    if base == *HYPERCORE_SPOT_HYPE_ASSET_ID {
        Some(HYPE_USDC_MARKET_INDEX)
    } else if base == *HYPERCORE_SPOT_UBTC_ASSET_ID {
        Some(UBTC_USDC_MARKET_INDEX)
    } else {
        None
    }
}

fn find_direct_market<'a>(meta: &SpotMeta, from_token: &'a SpotToken, to_token: &'a SpotToken) -> Result<(u32, &'a SpotToken, SpotSide), SwapperError> {
    if let Some(market_index) = shipped_market_index(from_token, to_token) {
        return Ok((market_index, from_token, SpotSide::Sell));
    }
    if let Some(market_index) = shipped_market_index(to_token, from_token) {
        return Ok((market_index, to_token, SpotSide::Buy));
    }
    for market in meta.universe.iter().filter(|m| m.tokens.len() == 2) {
        if market.tokens[0] == from_token.index && market.tokens[1] == to_token.index {
            return Ok((market.index, from_token, SpotSide::Sell));
        }
        if market.tokens[0] == to_token.index && market.tokens[1] == from_token.index {
            return Ok((market.index, to_token, SpotSide::Buy));
        }
    }
    Err(SwapperError::NotSupportedAsset)
}

fn compute_actual_from(use_max_amount: bool, amount: &str, decimals: u32) -> Result<Option<BigUint>, NumberFormatterError> {
    if !use_max_amount {
        return Ok(None);
    }
    BigNumberFormatter::value_from_amount_biguint(amount, decimals).map(Some)
}

#[derive(Debug)]
pub struct HyperCoreSpot {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    spot_meta_cache: Cache<(), SpotMeta>,
}

impl HyperCoreSpot {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperliquid),
            rpc_provider,
            spot_meta_cache: Cache::new(HOUR),
        }
    }

    fn client(&self) -> Result<HyperCoreClient<RpcClient>, SwapperError> {
        Ok(HyperCoreClient::new(create_rpc_client(self.rpc_provider.clone(), Chain::HyperCore)?))
    }

    async fn get_spot_meta(&self, client: &HyperCoreClient<RpcClient>) -> Result<SpotMeta, SwapperError> {
        if let Some(meta) = self.spot_meta_cache.get(&()) {
            return Ok(meta);
        }
        let meta = client.get_spot_meta().await.map_err(SwapperError::compute_quote_error)?;
        self.spot_meta_cache.put((), meta.clone());
        Ok(meta)
    }

    fn spot_token<'a>(&self, meta: &'a SpotMeta, asset: &'a SwapperQuoteAsset) -> Result<&'a SpotToken, SwapperError> {
        let asset_id = asset.asset_id();
        let components = asset_id.token_components().or_else(|| if asset_id == HYPERCORE_HYPE.id { HYPERCORE_SPOT_HYPE.id.token_components() } else { None });

        let (symbol, contract, index) = components.ok_or(SwapperError::NotSupportedAsset)?;
        let token = meta.tokens.iter().find(|token| token.name == symbol).ok_or(SwapperError::NotSupportedAsset)?;

        if let Some(contract) = contract
            && token.token_id != contract
        {
            return Err(SwapperError::NotSupportedAsset);
        }
        if let Some(index) = index
            && token.index != index
        {
            return Err(SwapperError::NotSupportedAsset);
        }

        Ok(token)
    }
}

#[async_trait]
impl Swapper for HyperCoreSpot {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::Assets(Chain::HyperCore, vec![HYPERCORE_SPOT_USDC.id.clone(), HYPERCORE_SPOT_UBTC.id.clone()])]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Flexible
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let client = self.client()?;
        let meta = self.get_spot_meta(&client).await?;
        let from_token = self.spot_token(&meta, &request.from_asset)?;
        let to_token = self.spot_token(&meta, &request.to_asset)?;

        let amount_in = BigNumberFormatter::decimal_value(&request.value, request.from_asset.decimals)?;
        if amount_in <= Decimal::zero() {
            return Err(SwapperError::ComputeQuoteError("amount must be greater than zero".into()));
        }

        let (market_index, base_token, side) = find_direct_market(&meta, from_token, to_token)?;
        let coin = format!("@{market_index}");
        let orderbook = client.get_spot_orderbook(&coin).await.map_err(SwapperError::compute_quote_error)?;
        if orderbook.levels.len() < 2 {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let (raw_output, base_limit_price, size_rounded, actual_from_value) = match side {
            SpotSide::Sell => {
                let rounded_input = round_size_down(&amount_in, base_token.sz_decimals);
                if rounded_input <= Decimal::zero() {
                    return Err(SwapperError::ComputeQuoteError("amount too small after rounding".into()));
                }
                let result = simulate_sell(&rounded_input, &orderbook.levels[0])?;
                let actual_from = compute_actual_from(request.options.use_max_amount, &format_decimal(&rounded_input), request.from_asset.decimals)?;
                (result.amount_out, result.limit_price, rounded_input, actual_from)
            }
            SpotSide::Buy => {
                let result = simulate_buy(&amount_in, &orderbook.levels[1])?;
                let rounded_output = round_size_down(&result.amount_out, base_token.sz_decimals);
                if rounded_output <= Decimal::zero() {
                    return Err(SwapperError::ComputeQuoteError("output too small after rounding".into()));
                }
                let actual_from = compute_actual_from(request.options.use_max_amount, &format_decimal(&(&rounded_output * &result.limit_price)), request.from_asset.decimals)?;
                (rounded_output.clone(), result.limit_price, rounded_output, actual_from)
            }
        };

        let output_amount = match side {
            SpotSide::Sell => {
                let builder_fee = i64::from(client.config.max_builder_fee_bps);
                let scale = i64::from(BUILDER_FEE_RATE_SCALE);
                &raw_output * Decimal::from(scale - builder_fee) / Decimal::from(scale)
            }
            SpotSide::Buy => raw_output,
        };

        let token_decimals = to_token.wei_decimals;

        let token_units = BigNumberFormatter::value_from_amount_biguint(format_decimal(&output_amount), token_decimals).map_err(|error| SwapperError::ComputeQuoteError(format!("{}: {error}", INVALID_AMOUNT)))?;
        let scaled_units = scale_units(token_units, token_decimals, request.to_asset.decimals)?;
        let to_value = scaled_units;

        let price_decimals = 8u32.saturating_sub(base_token.sz_decimals);
        let limit_price = limit_price_with_slippage(&base_limit_price, side, request.options.slippage.bps, price_decimals)?;
        if &size_rounded * &limit_price < Decimal::from(i64::from(MIN_ORDER_VALUE_USD)) {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }
        let limit_price = BigNumberFormatter::decimal_to_string(&limit_price, price_decimals);

        let order_size = format_order_size(&size_rounded, base_token.sz_decimals);

        let asset_index = spot_asset_index(market_index);

        let from_value = actual_from_value.unwrap_or_else(|| request.value.clone());

        let order = make_market_order(
            asset_index,
            side.is_buy(),
            &limit_price,
            &order_size,
            false,
            Some(Builder {
                builder_address: client.config.builder_address.clone(),
                fee: client.config.max_builder_fee_bps,
            }),
        );
        Ok(Quote {
            from_value,
            ..Quote::from_route(request, &self.provider, to_value, request.options.slippage.bps, &order, None)?
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let order: PlaceOrder = quote.route_data()?;
        let order_json = serde_json::to_string(&order).map_err(SwapperError::transaction_error)?;

        Ok(SwapperQuoteData::new_contract(String::new(), quote.from_value.clone(), order_json, None, None))
    }
}

#[cfg(test)]
mod unit_tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use gem_hypercore::models::spot::SpotMarket;

    use crate::{
        Route,
        alien::mock::{MockFn, ProviderMock},
        testkit::mock_quote,
    };

    use super::*;

    #[tokio::test]
    async fn test_get_spot_meta_cache() {
        let requests = Arc::new(AtomicUsize::new(0));
        let requests_ref = requests.clone();
        let provider = Arc::new(ProviderMock {
            response: MockFn(Box::new(move |_| {
                requests_ref.fetch_add(1, Ordering::Relaxed);
                include_str!("../../../../../gem_hypercore/testdata/spot_meta_spot_swap.json").to_string()
            })),
        });
        let spot = HyperCoreSpot::new(provider);
        let client = spot.client().unwrap();

        spot.get_spot_meta(&client).await.unwrap();
        spot.get_spot_meta(&client).await.unwrap();

        assert_eq!(requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_find_direct_market() {
        let token = |asset: &primitives::Asset| {
            let (name, token_id, index) = asset.id.token_components().unwrap();
            SpotToken {
                name,
                wei_decimals: 8,
                index: index.unwrap(),
                token_id: token_id.unwrap(),
                sz_decimals: 2,
            }
        };
        let (usdc, hype, ubtc) = (token(&HYPERCORE_SPOT_USDC), token(&HYPERCORE_SPOT_HYPE), token(&HYPERCORE_SPOT_UBTC));
        let other = SpotToken {
            name: "TEST".to_string(),
            index: 1,
            token_id: "0x00000000000000000000000000000001".to_string(),
            ..usdc.clone()
        };
        let market = |tokens: [&SpotToken; 2], index: u32| SpotMarket {
            tokens: tokens.map(|token| token.index).to_vec(),
            name: format!("@{index}"),
            index,
        };
        let meta = SpotMeta {
            tokens: vec![],
            universe: vec![market([&usdc, &hype], 1), market([&hype, &usdc], 2), market([&ubtc, &usdc], 3), market([&other, &usdc], 4)],
        };
        let find = |from: &SpotToken, to: &SpotToken| find_direct_market(&meta, from, to).map(|(index, base, side)| (index, base.index, side.is_buy()));

        assert_eq!(find(&hype, &usdc), Ok((HYPE_USDC_MARKET_INDEX, hype.index, false)), "a shipped pair trades on its shipped market, whatever the node lists");
        assert_eq!(find(&usdc, &hype), Ok((HYPE_USDC_MARKET_INDEX, hype.index, true)));
        assert_eq!(find(&ubtc, &usdc), Ok((UBTC_USDC_MARKET_INDEX, ubtc.index, false)));
        assert_eq!(find(&usdc, &ubtc), Ok((UBTC_USDC_MARKET_INDEX, ubtc.index, true)));
        assert_eq!(find(&other, &usdc), Ok((4, other.index, false)), "any other pair trades on the market the node lists");
        assert_eq!(find(&usdc, &other), Ok((4, other.index, true)));
        assert_eq!(find(&hype, &ubtc), Err(SwapperError::NotSupportedAsset));
    }

    #[tokio::test]
    async fn test_an_order_worth_less_than_the_minimum_after_size_rounding_is_refused() {
        let spot = HyperCoreSpot::mock();
        let mut below = mock_quote(SwapperQuoteAsset::from(&*HYPERCORE_SPOT_USDC), SwapperQuoteAsset::from(&*HYPERCORE_HYPE));
        below.value = BigUint::from(1_000_000_000u64);
        let mut above = below.clone();
        above.value = BigUint::from(1_100_000_000u64);

        assert_eq!(spot.get_quote(&below).await.err(), Some(SwapperError::InputAmountError { min_amount: None }));
        let order: PlaceOrder = spot.get_quote(&above).await.unwrap().route_data().unwrap();
        assert_eq!((order.orders[0].price.as_str(), order.orders[0].size.as_str()), ("87.134", "0.12"));
    }

    #[tokio::test]
    async fn test_the_builder_fee_comes_off_a_sell_only() {
        let spot = HyperCoreSpot::mock();
        let mut buy = mock_quote(SwapperQuoteAsset::from(&*HYPERCORE_SPOT_USDC), SwapperQuoteAsset::from(&*HYPERCORE_HYPE));
        buy.value = BigUint::from(1_100_000_000u64);
        let mut sell = mock_quote(SwapperQuoteAsset::from(&*HYPERCORE_HYPE), SwapperQuoteAsset::from(&*HYPERCORE_SPOT_USDC));
        sell.value = BigUint::from(12_000_000u64);

        assert_eq!(spot.get_quote(&buy).await.unwrap().to_value, BigUint::from(12_000_000u64));
        assert_eq!(spot.get_quote(&sell).await.unwrap().to_value, BigUint::from(1_038_732_400u64));
    }

    #[tokio::test]
    async fn test_quote_data_uses_adjusted_from_value() {
        let spot = HyperCoreSpot::new(Arc::new(ProviderMock::new(String::new())));
        let mut quote = Quote::mock(Chain::HyperCore, None);
        quote.request.value = BigUint::from(11000000u64);
        quote.from_value = BigUint::from(10000000u64);
        quote.data.routes = vec![Route {
            input: quote.request.from_asset.asset_id(),
            output: quote.request.to_asset.asset_id(),
            route_data: serde_json::to_string(&make_market_order(10_001, false, "1", "1", false, None)).unwrap(),
        }];

        let data = spot.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(data.value, quote.from_value);
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use super::*;
    use crate::{hyperliquid::provider::spot::math::SPOT_ASSET_OFFSET, testkit::mock_quote};
    use primitives::swap::SwapQuoteDataType;

    async fn assert_spot_quote(from_asset: &primitives::Asset, to_asset: &primitives::Asset) {
        let spot = HyperCoreSpot::new(Arc::new(crate::NativeProvider::new()));

        let mut request = mock_quote(
            SwapperQuoteAsset::mock_with_asset_id(from_asset.id.clone(), &from_asset.symbol, from_asset.decimals),
            SwapperQuoteAsset::mock_with_asset_id(to_asset.id.clone(), &to_asset.symbol, to_asset.decimals),
        );
        request.value = BigUint::from(2000000000u64);

        let quote = spot.get_quote(&request).await.unwrap();

        let order: PlaceOrder = serde_json::from_str(&quote.data.routes[0].route_data).unwrap();
        assert_eq!(order.r#type, "order");
        assert!(order.orders[0].asset >= SPOT_ASSET_OFFSET);

        let quote_data = spot.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(quote.data.provider.id, SwapperProvider::Hyperliquid);
        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);

        let from_amount = BigNumberFormatter::decimal_value(&quote.from_value, quote.request.from_asset.decimals).unwrap();
        let to_amount = BigNumberFormatter::decimal_value(&quote.to_value, quote.request.to_asset.decimals).unwrap();

        assert!(!from_amount.is_zero());
        assert!(!to_amount.is_zero());

        println!(
            "HyperCoreSpot: {} {} -> {} {} (rate: {})",
            from_amount,
            quote.request.from_asset.symbol,
            to_amount,
            quote.request.to_asset.symbol,
            &to_amount / &from_amount
        );
    }

    #[tokio::test]
    async fn test_spot_quote_hype_usdc() {
        assert_spot_quote(&HYPERCORE_HYPE, &HYPERCORE_SPOT_USDC).await;
        assert_spot_quote(&HYPERCORE_SPOT_USDC, &HYPERCORE_HYPE).await;
    }

    #[tokio::test]
    async fn test_spot_quote_ubtc_usdc() {
        assert_spot_quote(&HYPERCORE_SPOT_UBTC, &HYPERCORE_SPOT_USDC).await;
        assert_spot_quote(&HYPERCORE_SPOT_USDC, &HYPERCORE_SPOT_UBTC).await;
    }
}
