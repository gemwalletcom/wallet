use pricer::{ChartClient, PriceClient};
use primitives::{AssetMarketPrice, AssetPrices, AssetPricesRequest, ChartPeriod, Charts, DEFAULT_FIAT_CURRENCY, FiatRate};
use rocket::{State, get, post, serde::json::Json};

use crate::params::{AssetIdParam, ChartPeriodParam, CurrencyParam};
use crate::responders::{ApiError, ApiResponse};

#[get("/prices/<asset_id>?<currency>")]
pub async fn get_price(asset_id: AssetIdParam, currency: CurrencyParam, price_client: &State<PriceClient>) -> Result<ApiResponse<AssetMarketPrice>, ApiError> {
    Ok(price_client.get_asset_price(&asset_id.0, currency.0.as_ref()).await?.into())
}

#[post("/prices", format = "json", data = "<request>")]
pub async fn get_assets_prices(request: Json<AssetPricesRequest>, price_client: &State<PriceClient>) -> Result<ApiResponse<AssetPrices>, ApiError> {
    let AssetPricesRequest { currency, asset_ids } = request.into_inner();
    let currency = currency.as_ref().map(|currency| currency.as_ref()).unwrap_or(DEFAULT_FIAT_CURRENCY);
    Ok(price_client.get_asset_prices(currency, asset_ids).await?.into())
}

#[get("/fiat_rates")]
pub async fn get_fiat_rates(price_client: &State<PriceClient>) -> Result<ApiResponse<Vec<FiatRate>>, ApiError> {
    Ok(price_client.get_fiat_rates()?.into())
}

#[get("/charts/<asset_id>?<period>&<currency>")]
pub async fn get_charts(
    asset_id: AssetIdParam,
    period: Option<ChartPeriodParam>,
    currency: CurrencyParam,
    charts_client: &State<ChartClient>,
    price_client: &State<PriceClient>,
) -> Result<ApiResponse<Charts>, ApiError> {
    let period = period.map(|p| p.0).unwrap_or(ChartPeriod::Day);

    let asset_id = asset_id.0;
    let currency = currency.0;
    let prices = charts_client.get_charts_prices(&asset_id, period, currency.as_ref()).await?;
    let asset_price = price_client.get_asset_price(&asset_id, currency.as_ref()).await?;

    let response = Charts {
        price: asset_price.price,
        market: asset_price.market,
        prices,
        market_caps: vec![],
        total_volumes: vec![],
    };

    Ok(response.into())
}
