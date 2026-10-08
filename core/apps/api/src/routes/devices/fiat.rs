use std::sync::Arc;

use axum::extract::State;
use primitives::{FiatAssets, FiatQuoteRequest, FiatQuoteUrl, FiatQuotes, FiatTransactionData};
use serde::Deserialize;
use services::fiat::FiatClient;

use crate::auth::device::{AuthenticatedDevice, AuthenticatedDeviceWallet};
use crate::error::{ApiError, fiat_error};
use crate::request::{AssetIdParam, ClientIp, CurrencyParam, FiatProviderIdParam, FiatQuoteTypeParam, Path, Query, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct QuotesQuery {
    amount: f64,
    #[serde(default)]
    currency: CurrencyParam,
    #[serde(default, deserialize_with = "lenient")]
    provider: Option<FiatProviderIdParam>,
}

pub async fn get_fiat_transactions(device: AuthenticatedDeviceWallet, State(client): State<Arc<FiatClient>>) -> Result<ApiResponse<Vec<FiatTransactionData>>, ApiError> {
    Ok(client.get_transactions_by_device_wallet_id(device.record.id, device.wallet_id).await?.into())
}

pub async fn get_fiat_assets(_device: AuthenticatedDevice, Path(quote_type): Path<FiatQuoteTypeParam>, State(client): State<Arc<FiatClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_quote_assets(quote_type.0).await?.into())
}

pub async fn get_fiat_quotes(
    device: AuthenticatedDeviceWallet,
    Path((quote_type, asset_id)): Path<(FiatQuoteTypeParam, AssetIdParam)>,
    Query(query): Query<QuotesQuery>,
    ClientIp(ip): ClientIp,
    State(client): State<Arc<FiatClient>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let ip_address = ip.to_string();
    let quote_request = FiatQuoteRequest {
        asset_id: asset_id.0,
        quote_type: quote_type.0,
        amount: query.amount,
        currency: query.currency.0.as_ref().to_string(),
        provider_id: query.provider.map(|provider| provider.0.id().to_string()),
        ip_address: ip_address.clone(),
    };
    let context = fiat::FiatDeviceContext::new(device.record.id, device.wallet_id, device.wallet_type, ip_address);
    let quotes = client.get_device_quotes(quote_request, &context).await.map_err(|error| fiat_error(error, device.record.device.locale.as_ref()))?;
    Ok(quotes.into())
}

pub async fn get_fiat_quote_url(device: AuthenticatedDeviceWallet, Path(quote_id): Path<String>, ClientIp(ip): ClientIp, State(client): State<Arc<FiatClient>>) -> Result<ApiResponse<FiatQuoteUrl>, ApiError> {
    let locale = device.record.device.locale.as_ref();
    let context = fiat::FiatDeviceContext::new(device.record.id, device.wallet_id, device.wallet_type, ip.to_string());
    let url = client.get_quote_url(&quote_id, &context, locale).await.map_err(|error| fiat_error(error, locale))?;
    Ok(url.into())
}
