use gem_client::{Target, build_path_with_query};
use primitives::{AssetId, Chain, ChartPeriod, FiatQuoteType};

#[derive(Clone, Debug)]
pub enum GemApiTarget {
    GetConfig,
    GetCharts { asset_id: AssetId, period: ChartPeriod },
    GetAsset { asset_id: AssetId },
    GetAssets { currency: Option<String> },
    GetSearchAssets { query: String, chains: Vec<Chain> },
    GetSearch { query: String, chains: Vec<Chain>, tags: Vec<String> },
    GetPrices,
    GetMarkets,
    GetFiatAssets { quote_type: FiatQuoteType },
    GetSwapAssets,
}

impl Target for GemApiTarget {
    fn path(&self) -> String {
        match self {
            Self::GetConfig => "/v1/config".to_string(),
            Self::GetCharts { asset_id, period } => format!("/v1/charts/{asset_id}?period={}", period.as_ref()),
            Self::GetAsset { asset_id } => format!("/v1/assets/{asset_id}"),
            Self::GetAssets { currency } => match currency {
                Some(currency) => format!("/v1/assets?currency={currency}"),
                None => "/v1/assets".to_string(),
            },
            Self::GetSearchAssets { query, chains } => build_path_with_query("/v1/assets/search", &[("query", query.as_str()), ("chains", &join_chains(chains))]),
            Self::GetSearch { query, chains, tags } => {
                build_path_with_query("/v1/search", &[("query", query.as_str()), ("chains", &join_chains(chains)), ("tags", &tags.join(","))])
            }
            Self::GetPrices => "/v1/prices".to_string(),
            Self::GetMarkets => "/v1/markets".to_string(),
            Self::GetFiatAssets { quote_type } => format!("/v1/fiat/assets/{}", quote_type.as_ref()),
            Self::GetSwapAssets => "/v1/swap/assets".to_string(),
        }
    }
}

fn join_chains(chains: &[Chain]) -> String {
    chains.iter().map(Chain::as_ref).collect::<Vec<_>>().join(",")
}
