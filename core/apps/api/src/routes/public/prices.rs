use std::sync::Arc;

use axum::extract::State;
use primitives::currency::Currency;
use primitives::{AssetMarketPrice, AssetPrices, AssetPricesRequest, ChartPeriod, Charts, FiatRate};
use serde::Deserialize;
use services::prices::{ChartClient, PriceClient};

use crate::error::ApiError;
use crate::request::{AssetIdParam, ChartPeriodParam, CurrencyParam, CurrencyQuery, Json, Path, Query, lenient};
use crate::response::ApiResponse;

use crate::routes::fiat_rates::filter_fiat_rates_v1;

#[derive(Deserialize)]
pub struct ChartsQuery {
    #[serde(default, deserialize_with = "lenient")]
    period: Option<ChartPeriodParam>,
    #[serde(default)]
    currency: CurrencyParam,
}

pub async fn get_price(Path(asset_id): Path<AssetIdParam>, Query(query): Query<CurrencyQuery>, State(prices): State<Arc<PriceClient>>) -> Result<ApiResponse<AssetMarketPrice>, ApiError> {
    Ok(prices.get_asset_price(&asset_id.0, &query.currency.0).await?.into())
}

pub async fn get_assets_prices(State(prices): State<Arc<PriceClient>>, Json(request): Json<AssetPricesRequest>) -> Result<ApiResponse<AssetPrices>, ApiError> {
    let AssetPricesRequest { currency, asset_ids } = request;
    let currency = currency.unwrap_or(Currency::USD);
    Ok(prices.get_asset_prices(currency, asset_ids).await?.into())
}

pub async fn get_fiat_rates(State(prices): State<Arc<PriceClient>>) -> Result<ApiResponse<Vec<FiatRate>>, ApiError> {
    Ok(filter_fiat_rates_v1(prices.get_fiat_rates().await?).into())
}

pub async fn get_charts(Path(asset_id): Path<AssetIdParam>, Query(query): Query<ChartsQuery>, State(charts): State<Arc<ChartClient>>, State(prices): State<Arc<PriceClient>>) -> Result<ApiResponse<Charts>, ApiError> {
    let period = query.period.map(|period| period.0).unwrap_or(ChartPeriod::Day);
    let asset_id = asset_id.0;
    let currency = query.currency.0;
    let chart_prices = charts.get_charts_prices(&asset_id, period, &currency).await?;
    let asset_price = prices.get_asset_price(&asset_id, &currency).await?;

    Ok(Charts {
        price: asset_price.price,
        market: asset_price.market,
        prices: chart_prices,
        market_caps: vec![],
        total_volumes: vec![],
    }
    .into())
}
