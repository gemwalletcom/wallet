use std::{collections::HashMap, error::Error};

use gem_client::ReqwestClient;
use primitives::FiatProviderName;
use reqwest::Method;

use super::models::{Asset, CheckoutOrder, Country, CreateOrderRequest, FiatCurrency, Order, PAYMENT_METHOD_CARD, Quote};

const BUY_ORDER_TYPE: &str = "buy";

pub struct BanxaClient {
    client: ReqwestClient,
    redirect_url: String,
    partner: String,
    pub(super) webhook_secret_key: String,
}

impl BanxaClient {
    pub const NAME: FiatProviderName = FiatProviderName::Banxa;

    pub fn new(client: ReqwestClient, redirect_url: String, partner: String, api_key: String, webhook_secret_key: String) -> Self {
        Self {
            client: client.with_default_headers(HashMap::from([("x-api-key".to_string(), api_key)])),
            redirect_url,
            partner,
            webhook_secret_key,
        }
    }

    pub async fn get_assets_buy(&self) -> Result<Vec<Asset>, Box<dyn Error + Send + Sync>> {
        let path = format!("/{}/v2/crypto/{BUY_ORDER_TYPE}", self.partner);
        Ok(self.client.request(Method::GET, &path).send().await?.json().await?)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, Box<dyn Error + Send + Sync>> {
        let path = format!("/{}/v2/orders/{order_id}", self.partner);
        Ok(self.client.request(Method::GET, &path).send().await?.json().await?)
    }

    pub async fn get_quote_buy(&self, symbol: &str, chain: &str, fiat_currency: &str, fiat_amount: f64) -> Result<Quote, Box<dyn Error + Send + Sync>> {
        let fiat_amount = fiat_amount.to_string();
        let query = vec![
            ("paymentMethodId", PAYMENT_METHOD_CARD),
            ("crypto", symbol),
            ("blockchain", chain),
            ("fiat", fiat_currency),
            ("fiatAmount", fiat_amount.as_str()),
        ];
        let path = format!("/{}/v2/quotes/buy", self.partner);
        Ok(self.client.request(Method::GET, &path).query(&query).send().await?.json().await?)
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, Box<dyn Error + Send + Sync>> {
        let path = format!("/{}/v2/countries", self.partner);
        Ok(self.client.request(Method::GET, &path).send().await?.json().await?)
    }

    pub async fn get_fiat_currencies_buy(&self) -> Result<Vec<FiatCurrency>, Box<dyn Error + Send + Sync>> {
        let path = format!("/{}/v2/fiats/{BUY_ORDER_TYPE}", self.partner);
        Ok(self.client.request(Method::GET, &path).send().await?.json().await?)
    }

    pub async fn create_buy_order(
        &self,
        quote_id: String,
        fiat_amount: f64,
        fiat_currency: String,
        symbol: String,
        network: String,
        wallet_address: String,
    ) -> Result<CheckoutOrder, Box<dyn Error + Send + Sync>> {
        let request = CreateOrderRequest::new(quote_id, symbol, fiat_currency, fiat_amount, network, wallet_address, self.redirect_url.clone());
        let path = format!("/{}/v2/buy", self.partner);
        let response = self.client.request(Method::POST, &path).json(&request).send().await?.error_for_status()?;
        response.json().await.map_err(|e| e.into())
    }
}
