use super::models::{Assets, PaybisQuote, PaybisResponse, QuoteRequest, Request, RequestResponse, SellAssets};
use super::target::PaybisTarget;
use crate::rsa_signature::generate_rsa_pss_signature;
use gem_client::{ClientExt, ReqwestClient};
use primitives::FiatProviderName;
use serde::Serialize;
use std::collections::HashMap;
use url::Url;

const PAYBIS_WIDGET_URL: &str = "https://widget.paybis.com";

pub struct PaybisClient {
    client: ReqwestClient,
    private_key: String,
}

impl PaybisClient {
    pub const NAME: FiatProviderName = FiatProviderName::Paybis;

    pub fn new(client: ReqwestClient, api_key: String, private_key: String) -> Self {
        Self {
            client: client.with_default_headers(HashMap::from([("authorization".to_string(), api_key)])),
            private_key,
        }
    }

    async fn signed_post<B: Serialize + Send + Sync, T: serde::de::DeserializeOwned + Send>(
        &self,
        target: PaybisTarget,
        body: &B,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let signature = generate_rsa_pss_signature(&self.private_key, &serde_json::to_string(body)?)?;
        self.client
            .post::<_, PaybisResponse<T>>(target, body)
            .headers(HashMap::from([("X-Request-Signature".to_string(), signature)]))
            .await?
            .into()
    }

    pub async fn get_buy_quote(&self, crypto_currency: String, fiat_currency: String, fiat_amount: f64) -> Result<PaybisQuote, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = QuoteRequest {
            amount: fiat_amount.to_string(),
            direction_change: "from".to_string(),
            is_received_amount: false,
            currency_code_from: fiat_currency,
            currency_code_to: crypto_currency,
        };

        self.signed_post(PaybisTarget::Quote, &request_body).await
    }

    pub async fn get_sell_quote(&self, crypto_currency: String, fiat_currency: String, fiat_amount: f64) -> Result<PaybisQuote, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = QuoteRequest {
            amount: fiat_amount.to_string(),
            direction_change: "to".to_string(),
            is_received_amount: true,
            currency_code_from: crypto_currency,
            currency_code_to: fiat_currency,
        };

        self.signed_post(PaybisTarget::Quote, &request_body).await
    }

    async fn get_assets(&self, flow: &str) -> Result<Assets, Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .get::<PaybisResponse<Assets>>(PaybisTarget::CurrencyPairs { flow: flow.to_string() })
            .await?
            .into()
    }

    pub async fn get_buy_assets(&self) -> Result<Assets, Box<dyn std::error::Error + Send + Sync>> {
        self.get_assets("buy-crypto").await
    }

    pub async fn get_sell_assets(&self) -> Result<SellAssets, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.client.get(PaybisTarget::SellCurrencyPairs).await?)
    }

    pub async fn create_request(&self, request_body: Request) -> Result<RequestResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.signed_post(PaybisTarget::Request, &request_body).await
    }

    pub async fn get_redirect_url(
        &self,
        wallet_address: &str,
        from_currency: &str,
        to_currency: &str,
        quote_id: &str,
        is_buy: bool,
        user_ip: &str,
        locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = if is_buy {
            Request::new_buy(
                wallet_address.to_owned(),
                wallet_address.to_owned(),
                to_currency.to_string(),
                from_currency.to_string(),
                quote_id.to_string(),
                user_ip.to_string(),
                locale.to_string(),
            )
        } else {
            Request::new_sell(
                wallet_address.to_owned(),
                wallet_address.to_owned(),
                to_currency.to_string(),
                from_currency.to_string(),
                quote_id.to_string(),
                user_ip.to_string(),
                locale.to_string(),
            )
        };

        let response = self.create_request(request_body).await?;

        let mut url = Url::parse(PAYBIS_WIDGET_URL)?;
        url.query_pairs_mut().append_pair("requestId", &response.request_id);

        Ok(url.to_string())
    }
}
