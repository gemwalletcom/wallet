use cacher::{CacheKey, CacherClient};
use primitives::{FiatAssetSymbol, FiatQuote, FiatQuoteUrl, RequestError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

use crate::model::FiatDeviceContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CachedFiatQuote {
    pub(crate) quote: FiatQuote,
    #[serde(flatten)]
    pub(crate) asset_symbol: FiatAssetSymbol,
    #[serde(default)]
    pub(crate) country_code: Option<String>,
    #[serde(default)]
    pub(crate) url: Option<FiatQuoteUrl>,
}

pub(crate) struct FiatCacherClient {
    cacher: CacherClient,
}

impl FiatCacherClient {
    pub(crate) fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub(crate) async fn set_quotes(&self, context: &FiatDeviceContext, cached_quotes: Vec<CachedFiatQuote>) -> Result<Vec<FiatQuote>, Box<dyn Error + Send + Sync>> {
        let scoped_quotes: Vec<_> = cached_quotes.into_iter().map(|quote| (Uuid::new_v4().to_string(), quote)).collect();
        let entries: Vec<_> = scoped_quotes
            .iter()
            .map(|(quote_id, quote)| (CacheKey::FiatQuote(context.device_id, context.wallet_id, &context.ip_address, quote_id), quote))
            .collect();
        self.cacher.set_values_cached(&entries).await?;

        Ok(scoped_quotes
            .into_iter()
            .map(|(quote_id, cached_quote)| FiatQuote {
                id: quote_id,
                ..cached_quote.quote
            })
            .collect())
    }

    pub(crate) async fn get_quote(&self, context: &FiatDeviceContext, quote_id: &str) -> Result<CachedFiatQuote, Box<dyn Error + Send + Sync>> {
        match self
            .cacher
            .get_cached_optional(CacheKey::FiatQuote(context.device_id, context.wallet_id, &context.ip_address, quote_id))
            .await?
        {
            Some(quote) => Ok(quote),
            None => Err(RequestError::Forbidden.into()),
        }
    }

    pub(crate) async fn set_quote_url(&self, context: &FiatDeviceContext, quote_id: &str, url: &FiatQuoteUrl) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = CacheKey::FiatQuote(context.device_id, context.wallet_id, &context.ip_address, quote_id);
        let Some(quote) = self.cacher.get_cached_optional::<CachedFiatQuote>(key).await? else {
            return Err(RequestError::Forbidden.into());
        };
        let quote = CachedFiatQuote { url: Some(url.clone()), ..quote };
        self.cacher
            .set_cached(CacheKey::FiatQuote(context.device_id, context.wallet_id, &context.ip_address, quote_id), &quote)
            .await
    }
}
