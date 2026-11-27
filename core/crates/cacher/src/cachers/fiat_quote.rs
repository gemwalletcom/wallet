use std::error::Error;

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

impl CacherClient {
    pub async fn set_quotes(&self, device_id: i32, wallet_id: i32, quotes: &[(String, CachedFiatQuote)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = quotes.iter().map(|(quote_id, quote)| (CacheKey::FiatQuote(device_id, wallet_id, quote_id), quote)).collect::<Vec<_>>();
        self.set_many(&entries).await?;
        Ok(())
    }

    pub async fn quote(&self, device_id: i32, wallet_id: i32, quote_id: &str) -> Result<Option<CachedFiatQuote>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::FiatQuote(device_id, wallet_id, quote_id)).await
    }
}
