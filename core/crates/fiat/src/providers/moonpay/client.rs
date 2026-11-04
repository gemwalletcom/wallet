use crate::model::{FiatProviderAsset, filter_token_id};

use super::mapper::{map_asset_chain, map_limits};
use super::models::{Asset, BuyQuoteQuery, Country, FiatCurrencyType, MoonPayBuyQuote, MoonPayIpAddress, MoonPayResponse, MoonPaySellQuote, SellQuoteQuery};
use super::target::MoonPayTarget;
use super::widget::MoonPayWidget;
use gem_client::{ClientError, ClientExt, ReqwestClient};
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{FiatProviderName, FiatQuoteType};

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

    fn api_key_query(&self) -> [(&'static str, &str); 1] {
        [("apiKey", self.api_key.as_str())]
    }

    pub async fn get_ip_address(&self, ip_address: &str) -> Result<MoonPayIpAddress, ClientError> {
        self.client.get(MoonPayTarget::IpAddress { ip_address: ip_address.to_string() }).query(&self.api_key_query()).await
    }

    pub async fn get_buy_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64) -> Result<MoonPayBuyQuote, Box<dyn std::error::Error + Send + Sync>> {
        let target = MoonPayTarget::BuyQuote {
            symbol,
            query: BuyQuoteQuery {
                base_currency_code: fiat_currency,
                base_currency_amount: fiat_amount.to_string(),
                are_fees_included: true,
            },
        };
        self.client.get::<MoonPayResponse<MoonPayBuyQuote>>(target).query(&self.api_key_query()).await?.into()
    }

    pub async fn get_sell_quote(&self, symbol: String, fiat_currency: String, fiat_amount: f64) -> Result<MoonPaySellQuote, Box<dyn std::error::Error + Send + Sync>> {
        let target = MoonPayTarget::SellQuote {
            symbol,
            query: SellQuoteQuery {
                quote_currency_code: fiat_currency,
                quote_currency_amount: fiat_amount.to_string(),
                are_fees_included: true,
            },
        };
        self.client.get::<MoonPayResponse<MoonPaySellQuote>>(target).query(&self.api_key_query()).await?.into()
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, ClientError> {
        self.client.get(MoonPayTarget::Currencies).await
    }

    pub async fn get_countries(&self) -> Result<Vec<Country>, ClientError> {
        self.client.get(MoonPayTarget::Countries).await
    }

    pub fn map_assets(currencies: Vec<Asset>) -> Vec<FiatProviderAsset> {
        let (fiat_currencies, assets): (Vec<Asset>, Vec<Asset>) = currencies.into_iter().partition(|currency| currency.currency_type == FiatCurrencyType::Fiat);
        let buy_limits = map_limits(&fiat_currencies, FiatQuoteType::Buy);
        let sell_limits = map_limits(&fiat_currencies, FiatQuoteType::Sell);
        assets.into_iter().filter_map(|asset| Self::map_asset(asset, &buy_limits, &sell_limits)).collect()
    }

    pub fn map_asset(asset: Asset, buy_limits: &[FiatAssetLimits], sell_limits: &[FiatAssetLimits]) -> Option<FiatProviderAsset> {
        let chain = map_asset_chain(asset.clone())?;
        let contract_address = match asset.metadata.as_ref().map(|m| m.network_code.as_str()) {
            Some("ripple") => asset.metadata.as_ref().and_then(|m| m.contract_address.as_deref().and_then(|s| s.split('.').next_back().map(String::from))),
            _ => asset.clone().metadata?.contract_address,
        };

        let token_id = filter_token_id(Some(chain), contract_address);

        if token_id.is_none() && !asset.is_base_asset.unwrap_or(false) {
            return None;
        }
        let enabled = !asset.is_suspended.unwrap_or(true);
        let is_sell_enabled = asset.is_sell_supported.unwrap_or(false);
        let sell_limits = if is_sell_enabled { sell_limits.to_vec() } else { vec![] };

        Some(FiatProviderAsset {
            id: asset.clone().code,
            provider: FiatProviderName::MoonPay,
            chain: Some(chain),
            token_id,
            symbol: asset.clone().code,
            network: asset.clone().metadata.map(|x| x.network_code),
            enabled,
            is_buy_enabled: !buy_limits.is_empty(),
            is_sell_enabled,
            unsupported_countries: Some(asset.unsupported_countries()),
            buy_limits: buy_limits.to_vec(),
            sell_limits,
        })
    }

    pub fn quote_redirect_url(&self, quote_type: FiatQuoteType, amount: f64, symbol: &str, wallet_address: &str, external_transaction_id: &str, ip_address: &str) -> Result<String, url::ParseError> {
        MoonPayWidget::new(self.api_key.clone(), self.secret_key.clone()).redirect_url(quote_type, amount, symbol, wallet_address, external_transaction_id, ip_address)
    }
}
