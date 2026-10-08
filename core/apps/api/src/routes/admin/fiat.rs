use std::sync::Arc;

use axum::extract::State;
use primitives::{FiatQuoteRequest, FiatQuotes};
use serde::Deserialize;
use services::fiat::FiatClient;

use crate::auth::api_client::FiatQuotesRead;
use crate::error::ApiError;
use crate::request::{AssetIdParam, ClientIp, CurrencyParam, FiatProviderIdParam, FiatQuoteTypeParam, Path, Query, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct FiatQuotesQuery {
    asset_id: AssetIdParam,
    amount: f64,
    #[serde(default)]
    currency: CurrencyParam,
    #[serde(default, deserialize_with = "lenient")]
    provider_id: Option<FiatProviderIdParam>,
    ip_address: Option<String>,
}

pub async fn get_fiat_quotes(
    _permission: FiatQuotesRead,
    Path(quote_type): Path<FiatQuoteTypeParam>,
    Query(query): Query<FiatQuotesQuery>,
    ClientIp(ip): ClientIp,
    State(client): State<Arc<FiatClient>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let quote_request = FiatQuoteRequest {
        asset_id: query.asset_id.0,
        quote_type: quote_type.0,
        amount: query.amount,
        currency: query.currency.0.as_ref().to_string(),
        provider_id: query.provider_id.map(|provider| provider.0.id().to_string()),
        ip_address: query.ip_address.unwrap_or_else(|| ip.to_string()),
    };
    Ok(client.get_quotes(quote_request).await?.into())
}
