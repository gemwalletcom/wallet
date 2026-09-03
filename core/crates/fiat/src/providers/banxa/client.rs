use std::{collections::HashMap, error::Error};

use gem_client::{ClientExt, ReqwestClient};
use primitives::FiatProviderName;

use super::models::{Asset, BuyQuoteQuery, CheckoutOrder, Country, CreateOrderRequest, FiatCurrency, Order, PAYMENT_METHOD_CARD, Quote};
use super::target::BanxaTarget;

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
        Ok(self.client.get(BanxaTarget::Assets { partner: self.partner.clone() }).await?)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, Box<dyn Error + Send + Sync>> {
        let target = BanxaTarget::Order {
            partner: self.partner.clone(),
            order_id: order_id.to_string(),
        };
        Ok(self.client.get(target).await?)
    }

    pub async fn get_quote_buy(&self, symbol: &str, chain: &str, fiat_currency: &str, fiat_amount: f64) -> Result<Quote, Box<dyn Error + Send + Sync>> {
        let target = BanxaTarget::BuyQuote {
            partner: self.partner.clone(),
            query: BuyQuoteQuery {
                payment_method_id: PAYMENT_METHOD_CARD,
                crypto: symbol.to_string(),
                blockchain: chain.to_string(),
                fiat: fiat_currency.to_string(),
                fiat_amount: fiat_amount.to_string(),
            },
        };
        Ok(self.client.get(target).await?)
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BanxaTarget::Countries { partner: self.partner.clone() }).await?)
    }

    pub async fn get_fiat_currencies_buy(&self) -> Result<Vec<FiatCurrency>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BanxaTarget::FiatCurrencies { partner: self.partner.clone() }).await?)
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
        Ok(self.client.post(BanxaTarget::CreateBuyOrder { partner: self.partner.clone() }, &request).await?)
    }
}
