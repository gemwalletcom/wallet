use crate::model::{FiatProviderAsset, filter_token_id};

use super::mapper::map_asset_chain;
use super::models::{Asset, Country, MoonPayBuyQuote, MoonPayIpAddress, MoonPayResponse, MoonPaySellQuote};
use super::widget::MoonPayWidget;
use gem_client::ReqwestClient;
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{FiatProviderName, FiatQuoteType, PaymentType};
use reqwest::Method;

#[derive(Clone)]
pub struct MoonPayClient {
    client: ReqwestClient,
    api_key: String,
    secret_key: String,
    pub(super) webhook_secret_key: String,
}

impl MoonPayClient {
    pub const NAME: FiatProviderName = FiatProviderName::MoonPay;

    pub fn new(client: ReqwestClient, api_key: String, secret_key: String, webhook_secret_key: String) -> Self {
        Self {
            client,
            api_key,
            secret_key,
            webhook_secret_key,
        }
    }

    pub async fn get_ip_address(&self, ip_address: &str) -> Result<MoonPayIpAddress, reqwest::Error> {
        self.client
            .request(Method::GET, "/v4/ip_address/")
            .query(&[("ipAddress", ip_address), ("apiKey", &self.api_key)])
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_buy_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64) -> Result<MoonPayBuyQuote, Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .request(Method::GET, &format!("/v3/currencies/{symbol}/buy_quote/"))
            .query(&[
                ("baseCurrencyCode", fiat_currency),
                ("baseCurrencyAmount", fiat_amount.to_string()),
                ("areFeesIncluded", "true".to_string()),
                ("apiKey", self.api_key.clone()),
            ])
            .send()
            .await?
            .json::<MoonPayResponse<MoonPayBuyQuote>>()
            .await?
            .into()
    }

    pub async fn get_sell_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64) -> Result<MoonPaySellQuote, Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .request(Method::GET, &format!("/v3/currencies/{symbol}/sell_quote/"))
            .query(&[
                ("quoteCurrencyCode", fiat_currency),
                ("quoteCurrencyAmount", fiat_amount.to_string()),
                ("areFeesIncluded", "true".to_string()),
                ("apiKey", self.api_key.clone()),
            ])
            .send()
            .await?
            .json::<MoonPayResponse<MoonPaySellQuote>>()
            .await?
            .into()
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, reqwest::Error> {
        self.client.request(Method::GET, "/v3/currencies").send().await?.json().await
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, reqwest::Error> {
        self.client.request(Method::GET, "/v3/countries").send().await?.json().await
    }

    pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
        let chain = map_asset_chain(asset.clone())?;
        let contract_address = match asset.metadata.as_ref().map(|m| m.network_code.as_str()) {
            Some("ripple") => asset
                .metadata
                .as_ref()
                .and_then(|m| m.contract_address.as_deref().and_then(|s| s.split('.').next_back().map(String::from))),
            _ => asset.clone().metadata?.contract_address,
        };

        let token_id = filter_token_id(Some(chain), contract_address);

        // Skip tokens without contract address (only base assets can have no token_id)
        if token_id.is_none() && !asset.is_base_asset.unwrap_or(false) {
            return None;
        }
        let enabled = !asset.is_suspended.unwrap_or(true);

        let payment_types = [PaymentType::Card, PaymentType::GooglePay, PaymentType::ApplePay];

        let buy_limits = if asset.min_buy_amount.is_some() || asset.max_buy_amount.is_some() {
            payment_types
                .iter()
                .map(|x| FiatAssetLimits {
                    currency: Currency::USD,
                    payment_type: x.clone(),
                    min_amount: asset.min_buy_amount,
                    max_amount: asset.max_buy_amount,
                })
                .collect()
        } else {
            vec![]
        };

        let sell_limits = if asset.min_sell_amount.is_some() || asset.max_sell_amount.is_some() {
            payment_types
                .iter()
                .map(|x| FiatAssetLimits {
                    currency: Currency::USD,
                    payment_type: x.clone(),
                    min_amount: asset.min_sell_amount,
                    max_amount: asset.max_sell_amount,
                })
                .collect()
        } else {
            vec![]
        };

        let is_buy_enabled = asset.min_buy_amount.is_some() || asset.max_buy_amount.is_some();
        let is_sell_enabled = asset.is_sell_supported.unwrap_or(false);

        Some(FiatProviderAsset {
            id: asset.clone().code,
            provider: FiatProviderName::MoonPay,
            chain: Some(chain),
            token_id,
            symbol: asset.clone().code,
            network: asset.clone().metadata.map(|x| x.network_code),
            enabled,
            is_buy_enabled,
            is_sell_enabled,
            unsupported_countries: Some(asset.unsupported_countries()),
            buy_limits,
            sell_limits,
        })
    }

    pub fn quote_redirect_url(&self, quote_type: FiatQuoteType, amount: f64, symbol: &str, wallet_address: &str, external_transaction_id: &str, ip_address: &str) -> String {
        MoonPayWidget::new(self.api_key.clone(), self.secret_key.clone()).redirect_url(quote_type, amount, symbol, wallet_address, external_transaction_id, ip_address)
    }
}
