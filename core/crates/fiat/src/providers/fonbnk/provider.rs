use std::error::Error;

use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData, FiatWebhook, PaymentType};

use crate::error::FiatQuoteError;
use crate::model::{FiatMapping, FiatProviderAsset};
use crate::{FiatProvider, FiatWebhookRequest};

use super::client::FonbnkClient;
use super::mapper::{map_assets, map_buy_crypto_amount, map_countries, map_crypto_leg, map_deposit_amount, map_fiat_leg, map_payment_methods, map_sell_crypto_amount, map_webhook};
use super::model::{FonbnkOrderLimits, FonbnkQuote, FonbnkQuoteLeg, FonbnkQuoteRequest, FonbnkWebhook};

#[async_trait]
impl FiatProvider for FonbnkClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn payment_methods(&self) -> Vec<PaymentType> {
        map_payment_methods()
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(map_assets(self.get_currencies().await?))
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(map_countries(&self.get_currencies().await?))
    }

    async fn parse_webhook(&self, request: FiatWebhookRequest) -> Result<FiatWebhook, Box<dyn Error + Send + Sync>> {
        self.verify_webhook(&request)?;
        let payload = serde_json::from_value::<FonbnkWebhook>(request.data).map_err(|_| FiatQuoteError::InvalidWebhook)?;
        map_webhook(payload).map_err(|_| FiatQuoteError::InvalidWebhook.into())
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let (fiat_leg, payment_type) = self.fiat_leg(&request_map.country_code, FiatQuoteType::Buy).await?;
        let crypto_leg = map_crypto_leg(&request_map.asset_symbol)?;
        let (limits, quote) = self.usd_quote(fiat_leg, crypto_leg, request.amount).await?;

        Ok(FiatQuoteResponse {
            crypto_amount: map_buy_crypto_amount(&limits, &quote, request.amount),
            quote_id: quote.quote_id,
            fiat_amount: request.amount,
            payment_methods: vec![payment_type],
        })
    }

    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let (fiat_leg, payment_type) = self.fiat_leg(&request_map.country_code, FiatQuoteType::Sell).await?;
        let crypto_leg = map_crypto_leg(&request_map.asset_symbol)?;
        let (limits, quote) = self.usd_quote(crypto_leg, fiat_leg, request.amount).await?;

        Ok(FiatQuoteResponse {
            crypto_amount: map_sell_crypto_amount(&limits, &quote, request.amount),
            quote_id: quote.quote_id,
            fiat_amount: request.amount,
            payment_methods: vec![payment_type],
        })
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        Ok(FiatQuoteUrl {
            redirect_url: self.widget.redirect_url(&data)?,
            provider_transaction_id: None,
        })
    }
}

impl FonbnkClient {
    async fn fiat_leg(&self, country_code: &str, quote_type: FiatQuoteType) -> Result<(FonbnkQuoteLeg, PaymentType), Box<dyn Error + Send + Sync>> {
        let currencies = self.get_currencies().await?;
        map_fiat_leg(&currencies, country_code, quote_type).ok_or_else(|| FiatQuoteError::RegionUnavailable.into())
    }

    async fn usd_quote(&self, deposit: FonbnkQuoteLeg, payout: FonbnkQuoteLeg, usd_amount: f64) -> Result<(FonbnkOrderLimits, FonbnkQuote), Box<dyn Error + Send + Sync>> {
        let limits = self.get_order_limits(&deposit, &payout).await?;
        let deposit = FonbnkQuoteLeg {
            amount: Some(map_deposit_amount(&limits, usd_amount)?),
            ..deposit
        };
        let quote = self.create_quote(&FonbnkQuoteRequest { deposit, payout }).await?;
        Ok((limits, quote))
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::{FiatAssetSymbol, FiatQuoteRequest};

    fn mapping(country_code: &str) -> FiatMapping {
        FiatMapping {
            asset_symbol: FiatAssetSymbol {
                symbol: "USDC".to_string(),
                network: Some("BASE".to_string()),
            },
            country_code: country_code.to_string(),
            ..FiatMapping::mock()
        }
    }

    #[tokio::test]
    async fn test_fonbnk_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = FiatQuoteRequest::mock();
        let quote = FiatProvider::get_quote_buy(&create_fonbnk_test_client(), request.clone(), mapping("NG")).await?;

        println!("Fonbnk buy quote: {:?}", quote);
        assert!(quote.crypto_amount > 0.0 && quote.crypto_amount < request.amount);
        Ok(())
    }

    #[tokio::test]
    async fn test_fonbnk_get_sell_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = FiatQuoteRequest::mock_sell();
        let quote = FiatProvider::get_quote_sell(&create_fonbnk_test_client(), request.clone(), mapping("KE")).await?;

        println!("Fonbnk sell quote: {:?}", quote);
        assert!(quote.crypto_amount > request.amount);
        Ok(())
    }

    #[tokio::test]
    async fn test_fonbnk_get_assets_and_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_fonbnk_test_client();
        let assets = FiatProvider::get_assets(&client).await?;
        let countries = FiatProvider::get_countries(&client).await?;

        println!("Found {} Fonbnk assets and {} countries", assets.len(), countries.len());
        assert!(assets.iter().any(|asset| asset.id == "BASE_USDC"));
        assert!(countries.iter().any(|country| country.alpha2 == "NG" && country.is_allowed));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBHOOK: &str = include_str!("../../../testdata/fonbnk/webhook_on_ramp.json");

    #[tokio::test]
    async fn test_parse_webhook_accepts_signed_order() {
        let webhook = FonbnkClient::mock().parse_webhook(FiatWebhookRequest::mock_fonbnk_signed(WEBHOOK)).await.unwrap();

        match webhook {
            FiatWebhook::Transaction(transaction) => assert_eq!(transaction.transaction_id, "4f6b2c1e-8d3a-4b5e-9f7a-2c1d0e9b8a76"),
            webhook => panic!("unexpected webhook: {webhook:?}"),
        }
    }

    #[tokio::test]
    async fn test_parse_webhook_rejects_unsigned_order() {
        assert!(FonbnkClient::mock().parse_webhook(FiatWebhookRequest::mock(WEBHOOK)).await.is_err());
    }
}
