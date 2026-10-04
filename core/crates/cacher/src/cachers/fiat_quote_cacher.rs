use std::error::Error;

use async_trait::async_trait;
use primitives::{FiatAssetSymbol, FiatQuote, FiatQuoteUrl};
use serde::{Deserialize, Serialize};

use crate::{CacheKey, CacherClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFiatQuote {
    pub quote: FiatQuote,
    #[serde(flatten)]
    pub asset_symbol: FiatAssetSymbol,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub url: Option<FiatQuoteUrl>,
}

#[async_trait]
pub trait FiatQuoteCacher: Send + Sync {
    async fn add_quotes(&self, device_id: i32, wallet_id: i32, quotes: &[(String, CachedFiatQuote)]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn quote(&self, device_id: i32, wallet_id: i32, quote_id: &str) -> Result<Option<CachedFiatQuote>, Box<dyn Error + Send + Sync>>;
    async fn set_quote(&self, device_id: i32, wallet_id: i32, quote_id: &str, quote: &CachedFiatQuote) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl FiatQuoteCacher for CacherClient {
    async fn add_quotes(&self, device_id: i32, wallet_id: i32, quotes: &[(String, CachedFiatQuote)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = quotes.iter().map(|(quote_id, quote)| (CacheKey::FiatQuote(device_id, wallet_id, quote_id), quote)).collect::<Vec<_>>();
        self.set_values_cached(&entries).await?;
        Ok(())
    }

    async fn quote(&self, device_id: i32, wallet_id: i32, quote_id: &str) -> Result<Option<CachedFiatQuote>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::FiatQuote(device_id, wallet_id, quote_id)).await
    }

    async fn set_quote(&self, device_id: i32, wallet_id: i32, quote_id: &str, quote: &CachedFiatQuote) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::FiatQuote(device_id, wallet_id, quote_id), quote).await
    }
}
