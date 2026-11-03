use std::error::Error;

use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatWebhook};
use url::Url;

use crate::error::FiatQuoteError;
use crate::model::{FiatMapping, FiatProviderAsset};
use crate::provider::generate_quote_id;
use crate::{FiatProvider, FiatWebhookRequest};

use super::client::TopperClient;
use super::mapper::{map_assets, map_countries, map_limits, map_on_ramp, map_webhook};
use super::model::TopperWebhook;

const BOOTSTRAP_TOKEN_PARAM: &str = "bt";

#[async_trait]
impl FiatProvider for TopperClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn Error + Send + Sync>> {
        let limits = map_limits(&self.get_onramp_payment_methods().await?);
        Ok(map_assets(self.get_onramp_assets().await?, &limits))
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(map_countries(self.get_onramp_countries().await?))
    }

    async fn parse_webhook(&self, request: FiatWebhookRequest) -> Result<FiatWebhook, Box<dyn Error + Send + Sync>> {
        self.verify_webhook(&request)?;
        let payload = serde_json::from_value::<TopperWebhook>(request.data).map_err(|_| FiatQuoteError::InvalidWebhook)?;
        Ok(map_webhook(payload)?)
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let flow = map_on_ramp(request.amount, &request.currency, &request_map.asset_symbol, None, &request_map.country_code)?;
        let simulation = self.simulate(flow).await?;
        Ok(FiatQuoteResponse::new(generate_quote_id(), request.amount, simulation.destination.amount))
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        Ok(FiatQuoteUrl {
            redirect_url: self.redirect_url(&data)?,
            provider_transaction_id: None,
        })
    }
}

impl TopperClient {
    fn redirect_url(&self, data: &FiatQuoteUrlData) -> Result<String, Box<dyn Error + Send + Sync>> {
        let flow = map_on_ramp(data.quote.fiat_amount, &data.quote.fiat_currency, &data.asset_symbol, Some(&data.wallet_address), &data.country_code)?;
        let mut url = Url::parse(&self.widget_url)?;
        url.query_pairs_mut().append_pair(BOOTSTRAP_TOKEN_PARAM, &self.token.sign(&data.quote.id, flow)?);
        Ok(url.into())
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::{FiatAssetSymbol, FiatQuoteRequest};

    #[tokio::test]
    async fn test_topper_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = FiatQuoteRequest::mock();
        let mapping = FiatMapping {
            asset_symbol: FiatAssetSymbol {
                symbol: "USDC".to_string(),
                network: Some("base".to_string()),
            },
            ..FiatMapping::mock()
        };
        let quote = FiatProvider::get_quote_buy(&create_topper_test_client(), request.clone(), mapping).await?;

        println!("Topper buy quote: {:?}", quote);
        assert!(quote.crypto_amount > 0.0 && quote.crypto_amount < request.amount);
        Ok(())
    }

    #[tokio::test]
    async fn test_topper_get_assets_and_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_topper_test_client();
        let assets = FiatProvider::get_assets(&client).await?;
        let countries = FiatProvider::get_countries(&client).await?;

        println!("Found {} Topper assets and {} countries", assets.len(), countries.len());
        assert!(assets.iter().any(|asset| asset.id == "base_USDC" && !asset.buy_limits.is_empty()));
        assert!(countries.iter().any(|country| country.alpha2 == "US"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use jsonwebtoken::jwk::Jwk;
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use primitives::{FiatAssetSymbol, FiatQuote};

    use super::super::testkit::TEST_SIGNING_PUBLIC_KEY;
    use super::*;

    const WEBHOOK: &str = include_str!("../../../testdata/topper/webhook_order_completed.json");
    const CLAIMS: &str = include_str!("../../../testdata/topper/bootstrap_token_claims.json");

    #[tokio::test]
    async fn test_parse_webhook() {
        let client = TopperClient::mock();

        assert_eq!(
            client.parse_webhook(FiatWebhookRequest::mock_topper_signed(WEBHOOK)).await.unwrap(),
            map_webhook(serde_json::from_str(WEBHOOK).unwrap()).unwrap()
        );
        assert!(client.parse_webhook(FiatWebhookRequest::mock(WEBHOOK)).await.is_err());
    }

    #[test]
    fn test_redirect_url() {
        let data = FiatQuoteUrlData {
            quote: FiatQuote {
                id: "3f2c1b9e-7a4d-4c8e-9b1f-5d6e7a8b9c0d".to_string(),
                fiat_amount: 100.0,
                fiat_currency: "USD".to_string(),
                ..FiatQuote::mock(FiatProviderName::Topper)
            },
            asset_symbol: FiatAssetSymbol {
                symbol: "ETH".to_string(),
                network: Some("ethereum".to_string()),
            },
            wallet_address: "0xb794f5ea0ba39494ce839613fffba74279579268".to_string(),
            ip_address: "192.0.2.1".to_string(),
            country_code: "US".to_string(),
            locale: "en".to_string(),
        };
        let url = Url::parse(&TopperClient::mock().redirect_url(&data).unwrap()).unwrap();
        let params: HashMap<String, String> = url.query_pairs().into_owned().collect();
        let key = DecodingKey::from_jwk(&serde_json::from_str::<Jwk>(TEST_SIGNING_PUBLIC_KEY).unwrap()).unwrap();
        let mut validation = Validation::new(Algorithm::ES256);
        validation.required_spec_claims.clear();
        let mut claims = decode::<serde_json::Value>(&params[BOOTSTRAP_TOKEN_PARAM], &key, &validation).unwrap().claims;
        claims["iat"] = 1_800_000_000.into();

        assert_eq!(url.origin().ascii_serialization(), "https://app.topperpay.com");
        assert_eq!(claims, serde_json::from_str::<serde_json::Value>(CLAIMS).unwrap());
    }
}
