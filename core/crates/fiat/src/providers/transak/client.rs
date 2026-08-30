use super::{
    mapper::map_widget_params,
    models::{Asset, Country, CreateWidgetUrlRequest, CreateWidgetUrlResponse, Data, FiatCurrency, Response, TokenResponse, TransakQuote, TransakResponse},
};
use gem_client::ReqwestClient;
use primitives::{AccessTokenCacher, FiatProviderName, FiatQuoteUrlData};
use reqwest::Method;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

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
        let mut query = vec![
            ("isBuyOrSell", quote_type.to_string()),
            ("fiatCurrency", fiat_currency.to_string()),
            ("cryptoCurrency", symbol.to_string()),
            ("network", network.to_string()),
            ("partnerApiKey", self.api_key.to_string()),
        ];
        if let Some(amount) = fiat_amount {
            query.push(("fiatAmount", amount.to_string()));
        }
        if let Some(amount) = crypto_amount {
            query.push(("cryptoAmount", amount.to_string()));
        }

        self.client
            .request(Method::GET, "/api/v1/pricing/public/quotes")
            .header("x-api-key", &self.api_key)
            .query(&query)
            .send()
            .await?
            .json::<TransakResponse<TransakQuote>>()
            .await?
            .into()
    }

    pub async fn create_widget_url(&self, params: HashMap<String, Value>, ip_address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        let request_body = CreateWidgetUrlRequest { params };

        let response: Data<CreateWidgetUrlResponse> = self
            .gateway
            .request(Method::POST, "/api/v2/auth/session")
            .header("access-token", &access_token)
            .header("x-api-key", &self.api_key)
            .header("x-user-ip", ip_address)
            .json(&request_body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data.widget_url)
    }

    pub async fn redirect_url(&self, quote: TransakQuote, data: &FiatQuoteUrlData) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.create_widget_url(map_widget_params(&self.api_key, &self.referrer_domain, quote, data), &data.ip_address)
            .await
    }

    pub async fn get_supported_assets(&self) -> Result<Response<Vec<Asset>>, reqwest::Error> {
        self.client
            .request(Method::GET, "/cryptocoverage/api/v1/public/crypto-currencies")
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_countries(&self) -> Result<Response<Vec<Country>>, reqwest::Error> {
        self.client.request(Method::GET, "/api/v2/countries").send().await?.json().await
    }

    pub async fn get_fiat_currencies(&self) -> Result<Response<Vec<FiatCurrency>>, reqwest::Error> {
        self.client.request(Method::GET, "/fiat/public/v1/currencies/fiat-currencies").send().await?.json().await
    }

    pub(super) async fn get_access_token(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.access_token_cacher.get_or_refresh(Box::pin(self.refresh_access_token())).await
    }

    async fn refresh_access_token(&self) -> Result<(String, Duration), Box<dyn Error + Send + Sync>> {
        let path = format!("/partners/api/v2/refresh-token?apiKey={}", self.api_key);
        let body = serde_json::json!({
            "apiKey": self.api_key
        });

        let response: Data<TokenResponse> = self
            .client
            .request(Method::POST, &path)
            .header("x-api-key", &self.api_key)
            .header("api-secret", &self.api_secret)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        let expires_in = Duration::from_secs(response.data.expires_at.saturating_sub(UNIX_EPOCH.elapsed()?.as_secs()));
        Ok((response.data.access_token, expires_in))
    }
}
