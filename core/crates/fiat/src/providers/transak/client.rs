use super::{
    mapper::map_widget_params,
    models::{
        Asset, Country, CreateWidgetUrlRequest, CreateWidgetUrlResponse, Data, FiatCurrency, QuoteQuery, RefreshTokenRequest, Response, TokenResponse, TransakQuote,
        TransakResponse,
    },
    target::{TransakGatewayTarget, TransakTarget},
};
use gem_client::{ClientError, ClientExt, ReqwestClient};
use primitives::{AccessTokenCacher, FiatProviderName, FiatQuoteUrlData};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

const API_KEY_HEADER: &str = "x-api-key";

pub struct TransakClient {
    client: ReqwestClient,
    gateway: ReqwestClient,
    api_key: String,
    api_secret: String,
    referrer_domain: String,
    access_token_cacher: Arc<dyn AccessTokenCacher>,
}

impl TransakClient {
    pub const NAME: FiatProviderName = FiatProviderName::Transak;

    pub fn new(
        client: ReqwestClient,
        gateway: ReqwestClient,
        api_key: String,
        api_secret: String,
        referrer_domain: String,
        access_token_cacher: Arc<dyn AccessTokenCacher>,
    ) -> Self {
        TransakClient {
            client,
            gateway,
            api_key,
            api_secret,
            referrer_domain,
            access_token_cacher,
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([(API_KEY_HEADER.to_string(), self.api_key.clone())])
    }

    pub async fn get_buy_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64, network: String) -> Result<TransakQuote, Box<dyn std::error::Error + Send + Sync>> {
        self.get_quote("buy", symbol, fiat_currency, Some(fiat_amount), None, network).await
    }

    pub async fn get_sell_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64, network: String) -> Result<TransakQuote, Box<dyn std::error::Error + Send + Sync>> {
        let buy_quote = self.get_buy_quote(symbol.clone(), fiat_currency.clone(), fiat_amount, network.clone()).await?;

        let sell_quote = self
            .get_quote(
                "sell",
                symbol.clone(),
                fiat_currency.clone(),
                None,
                Some(&buy_quote.crypto_amount.to_string()),
                network.clone(),
            )
            .await?;

        let crypto_amount = sell_quote.sell_crypto_amount(fiat_amount);
        self.get_quote("sell", symbol, fiat_currency, None, Some(&crypto_amount.to_string()), network).await
    }

    pub async fn get_quote(
        &self,
        quote_type: &str,
        symbol: String,
        fiat_currency: String,
        fiat_amount: Option<f64>,
        crypto_amount: Option<&str>,
        network: String,
    ) -> Result<TransakQuote, Box<dyn std::error::Error + Send + Sync>> {
        let query = QuoteQuery {
            is_buy_or_sell: quote_type.to_string(),
            fiat_currency,
            crypto_currency: symbol,
            network,
            partner_api_key: self.api_key.clone(),
            fiat_amount: fiat_amount.map(|amount| amount.to_string()),
            crypto_amount: crypto_amount.map(str::to_string),
        };
        self.client
            .get::<TransakResponse<TransakQuote>>(TransakTarget::Quotes { query })
            .headers(self.headers())
            .await?
            .into()
    }

    pub async fn create_widget_url(&self, params: HashMap<String, Value>, ip_address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        let request_body = CreateWidgetUrlRequest { params };
        let mut headers = self.headers();
        headers.insert("access-token".to_string(), access_token);
        headers.insert("x-user-ip".to_string(), ip_address.to_string());

        let response: Data<CreateWidgetUrlResponse> = self.gateway.post(TransakGatewayTarget::AuthSession, &request_body).headers(headers).await?;

        Ok(response.data.widget_url)
    }

    pub async fn redirect_url(&self, quote: TransakQuote, data: &FiatQuoteUrlData) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.create_widget_url(map_widget_params(&self.api_key, &self.referrer_domain, quote, data), &data.ip_address)
            .await
    }

    pub async fn get_supported_assets(&self) -> Result<Response<Vec<Asset>>, ClientError> {
        self.client.get(TransakTarget::CryptoCurrencies).await
    }

    pub async fn get_countries(&self) -> Result<Response<Vec<Country>>, ClientError> {
        self.client.get(TransakTarget::Countries).await
    }

    pub async fn get_fiat_currencies(&self) -> Result<Response<Vec<FiatCurrency>>, ClientError> {
        self.client.get(TransakTarget::FiatCurrencies).await
    }

    pub(super) async fn get_access_token(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.access_token_cacher.get_or_refresh(Box::pin(self.refresh_access_token())).await
    }

    async fn refresh_access_token(&self) -> Result<(String, Duration), Box<dyn Error + Send + Sync>> {
        let mut headers = self.headers();
        headers.insert("api-secret".to_string(), self.api_secret.clone());
        let body = RefreshTokenRequest { api_key: self.api_key.clone() };

        let response: Data<TokenResponse> = self
            .client
            .post(TransakTarget::RefreshToken { api_key: self.api_key.clone() }, &body)
            .headers(headers)
            .await?;
        let expires_in = Duration::from_secs(response.data.expires_at.saturating_sub(UNIX_EPOCH.elapsed()?.as_secs()));
        Ok((response.data.access_token, expires_in))
    }
}
