use rocket::{State, get, tokio::sync::Mutex};

use crate::admin::AdminAuthorized;
use crate::devices::FiatQuotesClient;
use crate::params::{AssetIdParam, CurrencyParam, FiatProviderIdParam, FiatQuoteTypeParam};
use crate::responders::{ApiError, ApiResponse};
use primitives::{FiatQuoteRequest, FiatQuotes};

#[get("/fiat/quotes/<quote_type>?<asset_id>&<amount>&<currency>&<provider_id>&<ip_address>")]
pub async fn get_fiat_quotes(
    _admin: AdminAuthorized,
    quote_type: FiatQuoteTypeParam,
    asset_id: AssetIdParam,
    amount: f64,
    currency: CurrencyParam,
    provider_id: Option<FiatProviderIdParam>,
    ip_address: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let quote_request = FiatQuoteRequest {
        asset_id: asset_id.0,
        quote_type: quote_type.0,
        amount,
        currency: currency.as_string(),
        provider_id: provider_id.map(|p| p.0.id().to_string()),
        ip_address: ip_address.map(str::to_string).unwrap_or_else(|| ip.to_string()),
    };
    let quotes = client.lock().await.get_quotes(quote_request).await?;
    Ok(quotes.into())
}
