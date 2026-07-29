use super::models::{Asset, CachedToken, Country, CreateWidgetUrlRequest, CreateWidgetUrlResponse, Data, FiatCurrency, Response, TokenResponse, TransakQuote, TransakResponse};
use primitives::{FiatProviderName, FiatQuoteType, FiatQuoteUrlData};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const TRANSAK_API_URL: &str = "https://api.transak.com";
const TRANSAK_API_GATEWAY_URL: &str = "https://api-gateway.transak.com";
const TOKEN_TTL_SECONDS: u64 = 3600;

#[derive(Debug, Clone)]
pub struct TransakClient {
    pub client: Client,
    pub api_key: String,
    pub api_secret: String,
    pub referrer_domain: String,
    cached_token: Arc<Mutex<Option<CachedToken>>>,
}

impl TransakClient {
    pub const NAME: FiatProviderName = FiatProviderName::Transak;

    pub fn new(client: Client, api_key: String, api_secret: String, referrer_domain: String) -> Self {
        TransakClient {
            client,
            api_key,
            api_secret,
            referrer_domain,
            cached_token: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(test)]
    pub(super) fn new_with_access_token(access_token: &str) -> Self {
        Self {
            client: gem_client::reqwest_client(),
            api_key: String::new(),
            api_secret: String::new(),
            referrer_domain: String::new(),
            cached_token: Arc::new(Mutex::new(Some(CachedToken::new(access_token.to_string(), TOKEN_TTL_SECONDS)))),
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
        let url = format!("{TRANSAK_API_URL}/api/v1/pricing/public/quotes");
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
            .get(url)
            .header("x-api-key", &self.api_key)
            .query(&query)
            .send()
            .await?
            .json::<TransakResponse<TransakQuote>>()
            .await?
            .into()
    }

    pub async fn create_widget_url(&self, params: HashMap<String, Value>, ip_address: &str) -> Result<String, reqwest::Error> {
        let access_token = self.get_access_token().await?;
        let url = format!("{TRANSAK_API_GATEWAY_URL}/api/v2/auth/session");

        let request_body = CreateWidgetUrlRequest { params };

        let response: Data<CreateWidgetUrlResponse> = self
            .client
            .post(&url)
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

    fn build_widget_params(&self, quote: TransakQuote, data: &FiatQuoteUrlData) -> HashMap<String, Value> {
        let sell_crypto_amount = quote.sell_crypto_amount(data.quote.fiat_amount);

        let mut params: HashMap<String, Value> = HashMap::new();
        params.insert("apiKey".to_string(), json!(self.api_key));
        params.insert("referrerDomain".to_string(), json!(self.referrer_domain));
        params.insert("partnerOrderId".to_string(), json!(data.quote.id));
        params.insert("fiatCurrency".to_string(), json!(quote.fiat_currency));
        params.insert("cryptoCurrencyCode".to_string(), json!(quote.crypto_currency));
        params.insert("network".to_string(), json!(quote.network));
        params.insert("disableWalletAddressForm".to_string(), json!(true));
        params.insert("walletAddress".to_string(), json!(data.wallet_address));

        match &data.quote.quote_type {
            FiatQuoteType::Buy => {
                params.insert("productsAvailed".to_string(), json!("BUY"));
                params.insert("fiatAmount".to_string(), json!(data.quote.fiat_amount));
            }
            FiatQuoteType::Sell => {
                params.insert("productsAvailed".to_string(), json!("SELL"));
                params.insert("cryptoAmount".to_string(), json!(sell_crypto_amount));
            }
        }

        params
    }

    pub async fn redirect_url(&self, quote: TransakQuote, data: &FiatQuoteUrlData) -> Result<String, reqwest::Error> {
        self.create_widget_url(self.build_widget_params(quote, data), &data.ip_address).await
    }

    pub async fn get_supported_assets(&self) -> Result<Response<Vec<Asset>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/cryptocoverage/api/v1/public/crypto-currencies");
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_countries(&self) -> Result<Response<Vec<Country>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/api/v2/countries");
        self.client.get(&url).send().await?.json().await
    }

    pub async fn get_fiat_currencies(&self) -> Result<Response<Vec<FiatCurrency>>, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/fiat/public/v1/currencies/fiat-currencies");
        self.client.get(&url).send().await?.json().await
    }

    pub(super) async fn get_access_token(&self) -> Result<String, reqwest::Error> {
        let mut token_guard = self.cached_token.lock().await;

        if let Some(cached) = token_guard.as_ref()
            && cached.is_valid()
        {
            return Ok(cached.access_token.clone());
        }

        let access_token = self.refresh_token_internal().await?;
        let cached = CachedToken::new(access_token.clone(), TOKEN_TTL_SECONDS);
        *token_guard = Some(cached);

        Ok(access_token)
    }

    async fn refresh_token_internal(&self) -> Result<String, reqwest::Error> {
        let url = format!("{TRANSAK_API_URL}/partners/api/v2/refresh-token?apiKey={}", self.api_key);
        let body = serde_json::json!({
            "apiKey": self.api_key
        });

        let response: Data<TokenResponse> = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("api-secret", &self.api_secret)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(response.data.access_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain, FiatAssetSymbol, FiatProvider, FiatQuote};

    #[test]
    fn test_build_widget_params_uses_stored_quote_id_as_partner_order_id() {
        let client = TransakClient::new_with_access_token("access_token");
        let data = FiatQuoteUrlData {
            quote: FiatQuote {
                id: "stored_quote_id".to_string(),
                asset: Asset::from_chain(Chain::Ethereum),
                provider: FiatProvider::mock(FiatProviderName::Transak),
                quote_type: FiatQuoteType::Buy,
                fiat_amount: 100.0,
                fiat_currency: "USD".to_string(),
                crypto_amount: 0.03,
                value: "30000000000000000".to_string(),
                latency: 0,
                payment_methods: vec![],
            },
            asset_symbol: FiatAssetSymbol {
                symbol: "ETH".to_string(),
                network: Some("ethereum".to_string()),
            },
            wallet_address: "0x123".to_string(),
            ip_address: "192.0.2.1".to_string(),
            locale: "en".to_string(),
        };
        let quote = TransakQuote {
            quote_id: "provider_quote_id".to_string(),
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            crypto_currency: "ETH".to_string(),
            crypto_amount: 0.03,
            network: "ethereum".to_string(),
            conversion_price: 0.0003,
            total_fee: 1.0,
        };

        let params = client.build_widget_params(quote, &data);

        assert_eq!(params.get("partnerOrderId"), Some(&json!("stored_quote_id")));
    }
}
