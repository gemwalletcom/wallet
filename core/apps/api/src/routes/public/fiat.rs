use std::sync::Arc;

use axum::extract::State;
use primitives::FiatAssets;
use services::fiat::FiatClient;

use crate::error::ApiError;
use crate::request::{FiatQuoteTypeParam, Path};
use crate::response::ApiResponse;

pub async fn get_fiat_assets(Path(quote_type): Path<FiatQuoteTypeParam>, State(client): State<Arc<FiatClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_quote_assets(quote_type.0).await?.into())
}
