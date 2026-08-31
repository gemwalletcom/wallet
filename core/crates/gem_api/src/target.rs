use primitives::{AssetId, Chain, ChartPeriod, FiatQuoteType};

use crate::method::GemApiMethod;

#[derive(Clone, Debug)]
pub enum GemApiTarget {
    GetConfig,
    GetCharts(AssetId, ChartPeriod),
    GetAsset(AssetId),
    GetAssets(Vec<AssetId>, Option<String>),
    GetSearchAssets { query: String, chains: Vec<Chain> },
    GetSearch { query: String, chains: Vec<Chain>, tags: Vec<String> },
    GetPrices,
    GetMarkets,
    GetFiatAssets(FiatQuoteType),
    GetSwapAssets,
}

impl GemApiTarget {
    pub fn method(&self) -> GemApiMethod {
        match self {
            Self::GetAssets(_, _) | Self::GetPrices => GemApiMethod::Post,
            Self::GetConfig
            | Self::GetCharts(_, _)
            | Self::GetAsset(_)
            | Self::GetSearchAssets { .. }
            | Self::GetSearch { .. }
            | Self::GetMarkets
            | Self::GetFiatAssets(_)
            | Self::GetSwapAssets => GemApiMethod::Get,
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::GetConfig => "/v1/config".to_string(),
            Self::GetCharts(asset_id, period) => format!("/v1/charts/{asset_id}?period={}", period.as_ref()),
            Self::GetAsset(asset_id) => format!("/v1/assets/{asset_id}"),
            Self::GetAssets(_, currency) => match currency {
                Some(currency) => format!("/v1/assets?currency={currency}"),
                None => "/v1/assets".to_string(),
            },
            Self::GetSearchAssets { query, chains } => format!("/v1/assets/search?query={query}&chains={}", join_chains(chains)),
            Self::GetSearch { query, chains, tags } => {
                format!("/v1/search?query={query}&chains={}&tags={}", join_chains(chains), tags.join(","))
            }
            Self::GetPrices => "/v1/prices".to_string(),
            Self::GetMarkets => "/v1/markets".to_string(),
            Self::GetFiatAssets(quote_type) => format!("/v1/fiat/assets/{}", quote_type.as_ref()),
            Self::GetSwapAssets => "/v1/swap/assets".to_string(),
        }
    }
}

fn join_chains(chains: &[Chain]) -> String {
    chains.iter().map(Chain::as_ref).collect::<Vec<_>>().join(",")
}
