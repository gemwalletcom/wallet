use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::{DateTime, Price};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PriceProvider {
    #[default]
    Coingecko,
    Pyth,
    Jupiter,
    DefiLlama,
    TonApi,
    Hypercore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceProviderConfig {
    pub provider: PriceProvider,
    pub enabled: bool,
    pub priority: i32,
}

pub fn ranked_prices<'a, T>(providers: &[PriceProviderConfig], rows: &'a [T], provider: impl Fn(&T) -> PriceProvider) -> Vec<&'a T> {
    let mut candidates: Vec<(&PriceProviderConfig, &T)> = providers.iter().filter(|p| p.enabled).filter_map(|p| rows.iter().find(|row| provider(row) == p.provider).map(|row| (p, row))).collect();
    candidates.sort_by_key(|(p, _)| p.priority);
    candidates.into_iter().map(|(_, row)| row).collect()
}

pub fn primary_price<'a, T>(providers: &[PriceProviderConfig], rows: &'a [T], max_age: Duration, price: impl Fn(&T) -> Price) -> Option<&'a T> {
    let cutoff = DateTime::now() - max_age;
    ranked_prices(providers, rows, |row| price(row).provider).into_iter().find(|row| price(row).updated_at >= cutoff)
}

impl PriceProvider {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn primary() -> Self {
        Self::Coingecko
    }

    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn has_candles(&self) -> bool {
        match self {
            Self::Coingecko => true,
            Self::Pyth | Self::Jupiter | Self::DefiLlama | Self::TonApi | Self::Hypercore => false,
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            Self::Coingecko => 0,
            Self::Pyth => 1,
            Self::Jupiter => 2,
            Self::DefiLlama => 4,
            Self::TonApi => 3,
            Self::Hypercore => 5,
        }
    }
}

impl fmt::Display for PriceProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}
