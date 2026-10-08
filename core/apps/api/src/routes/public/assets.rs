use std::sync::Arc;

use axum::extract::State;
use primitives::{AssetBasic, AssetFull, AssetId, SearchResponse};
use serde::Deserialize;
use services::assets::{AssetsClient, SearchClient, SearchRequest};
use services::prices::PriceClient;

use crate::error::ApiError;
use crate::request::{AssetIdParam, CurrencyQuery, Json, Path, Query, QueryLimitParam, SearchQueryParam, lenient};
use crate::response::ApiResponse;

const SEARCH_LIMIT: usize = 50;

#[derive(Deserialize)]
pub struct SearchParams {
    query: SearchQueryParam,
    chains: Option<String>,
    tags: Option<String>,
    #[serde(default)]
    limit: QueryLimitParam,
    #[serde(default, deserialize_with = "lenient")]
    offset: Option<usize>,
}

pub async fn get_asset(Path(asset_id): Path<AssetIdParam>, Query(query): Query<CurrencyQuery>, State(client): State<Arc<AssetsClient>>, State(prices): State<Arc<PriceClient>>) -> Result<ApiResponse<AssetFull>, ApiError> {
    let asset = client.get_asset_full(&asset_id.0).await?;
    let rate = prices.get_fiat_rate(&query.currency.0).await?.rate;
    Ok(asset.with_rate(rate).into())
}

pub async fn get_assets(Query(query): Query<CurrencyQuery>, State(client): State<Arc<AssetsClient>>, State(prices): State<Arc<PriceClient>>, Json(asset_ids): Json<Vec<AssetId>>) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let rate = prices.get_fiat_rate(&query.currency.0).await?.rate;
    Ok(client.get_assets(asset_ids, rate).await?.into())
}

pub async fn get_assets_search(Query(params): Query<SearchParams>, State(client): State<Arc<SearchClient>>) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let request = SearchRequest::new(&params.query.0, params.chains.as_deref(), params.tags.as_deref(), params.limit.0, params.offset);
    Ok(client.get_assets_search(&request).await?.into())
}

pub async fn get_search(Query(params): Query<SearchParams>, State(client): State<Arc<SearchClient>>) -> Result<ApiResponse<SearchResponse>, ApiError> {
    let request = SearchRequest::new(&params.query.0, params.chains.as_deref(), params.tags.as_deref(), params.limit.0.min(SEARCH_LIMIT), params.offset);

    let lists = async { if request.should_search_lists() { client.get_asset_lists_search(&request).await } else { Ok(vec![]) } };
    let nfts = async { if request.has_tag_filter() { Ok(vec![]) } else { client.get_nfts_search(&request).await } };
    let (assets, lists, perpetuals, nfts) = futures::try_join!(client.get_assets_search(&request), lists, client.get_perpetuals_search(&request), nfts,)?;

    Ok(SearchResponse { assets, perpetuals, nfts, lists }.into())
}
