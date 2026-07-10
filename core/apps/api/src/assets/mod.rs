pub mod client;
mod filter;
mod model;

use crate::params::{AssetIdParam, CurrencyParam};
use crate::responders::{ApiError, ApiResponse};
pub use client::{AssetsClient, SearchClient};
use model::SearchParams;
pub use model::SearchRequest;
use pricer::PriceClient;
use primitives::{AssetBasic, AssetFull, AssetId, SearchResponse};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/assets/<asset_id>?<currency>")]
pub async fn get_asset(
    asset_id: AssetIdParam,
    currency: Option<CurrencyParam>,
    client: &State<Mutex<AssetsClient>>,
    price_client: &State<Mutex<PriceClient>>,
) -> Result<ApiResponse<AssetFull>, ApiError> {
    let asset = client.lock().await.get_asset_full(&asset_id.0)?;
    let currency = currency.unwrap_or_default().0;
    let rate = price_client.lock().await.get_fiat_rate(currency.as_ref())?.rate;
    Ok(asset.with_rate(rate).into())
}

#[post("/assets?<currency>", format = "json", data = "<asset_ids>")]
pub async fn get_assets(
    asset_ids: Json<Vec<AssetId>>,
    currency: Option<CurrencyParam>,
    client: &State<Mutex<AssetsClient>>,
    price_client: &State<Mutex<PriceClient>>,
) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let currency = currency.unwrap_or_default().0;
    let rate = price_client.lock().await.get_fiat_rate(currency.as_ref())?.rate;

    Ok(client.lock().await.get_assets(asset_ids.0, rate)?.into())
}

#[get("/assets/search?<params..>")]
pub async fn get_assets_search(params: SearchParams<'_>, client: &State<Mutex<SearchClient>>) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let request = SearchRequest::new(&params.query.0, params.chains, params.tags, params.limit, params.offset);
    Ok(client.lock().await.get_assets_search(&request).await?.into())
}

#[get("/search?<params..>")]
pub async fn get_search(params: SearchParams<'_>, client: &State<Mutex<SearchClient>>) -> Result<ApiResponse<SearchResponse>, ApiError> {
    let request = SearchRequest::new(&params.query.0, params.chains, params.tags, params.limit, params.offset);

    let search_client = client.lock().await;
    let assets = search_client.get_assets_search(&request).await?;
    let lists = if request.should_search_lists() {
        search_client.get_asset_lists_search(&request).await?
    } else {
        vec![]
    };
    let perpetuals = search_client.get_perpetuals_search(&request).await?;
    let nfts = if request.has_tag_filter() {
        vec![]
    } else {
        search_client.get_nfts_search(&request).await?
    };

    Ok(SearchResponse { assets, perpetuals, nfts, lists }.into())
}
