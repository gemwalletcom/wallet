use std::error::Error;

use cacher::{CacheKey, CacherClient};
use fiat::FiatDeviceContext;
use primitives::{FiatAssetSymbol, FiatQuote, FiatQuoteUrl, RequestError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CachedFiatQuote {
    pub(super) quote: FiatQuote,
    #[serde(flatten)]
    pub(super) asset_symbol: FiatAssetSymbol,
    #[serde(default)]
    pub(super) country_code: Option<String>,
    #[serde(default)]
    pub(super) url: Option<FiatQuoteUrl>,
}

pub(super) struct FiatCacherClient {
    cacher: CacherClient,
}

impl FiatCacherClient {
    pub(super) fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub(super) async fn set_quotes(&self, context: &FiatDeviceContext, cached_quotes: Vec<CachedFiatQuote>) -> Result<Vec<FiatQuote>, Box<dyn Error + Send + Sync>> {
        let scoped_quotes: Vec<_> = cached_quotes.into_iter().map(|quote| (Uuid::new_v4().to_string(), quote)).collect();
        let entries: Vec<_> = scoped_quotes.iter().map(|(quote_id, quote)| (quote_key(context, quote_id), quote)).collect();
        self.cacher.set_values_cached(&entries).await?;

        Ok(scoped_quotes.into_iter().map(|(quote_id, cached_quote)| FiatQuote { id: quote_id, ..cached_quote.quote }).collect())
    }

    pub(super) async fn get_quote(&self, context: &FiatDeviceContext, quote_id: &str) -> Result<CachedFiatQuote, Box<dyn Error + Send + Sync>> {
        match self.cacher.get_cached_optional(quote_key(context, quote_id)).await? {
            Some(quote) => Ok(quote),
            None => Err(RequestError::Forbidden.into()),
        }
    }

    pub(super) async fn set_quote_url(&self, context: &FiatDeviceContext, quote_id: &str, url: &FiatQuoteUrl) -> Result<(), Box<dyn Error + Send + Sync>> {
        let quote = self.get_quote(context, quote_id).await?;
        let quote = CachedFiatQuote { url: Some(url.clone()), ..quote };
        self.cacher.set_cached(quote_key(context, quote_id), &quote).await
    }
}

fn quote_key<'a>(context: &'a FiatDeviceContext, quote_id: &'a str) -> CacheKey<'a> {
    CacheKey::FiatQuote(context.device_id, context.wallet_id, quote_id)
}
