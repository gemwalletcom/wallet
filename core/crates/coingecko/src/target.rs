use gem_client::{Target, build_path_with_query};

use crate::model::{CoinMarketsQuery, CoinQuery, CointListQuery, MarketChartQuery};

#[derive(Clone, Debug)]
pub enum CoinGeckoTarget {
    Global,
    SearchTrending,
    TopGainersLosers,
    CoinList { query: CointListQuery },
    CoinCategories,
    NewCoins,
    CoinMarkets { query: CoinMarketsQuery },
    Coin { id: String, query: CoinQuery },
    CoinByContract { platform_id: String, contract_address: String },
    ExchangeRates,
    MarketChart { id: String, query: MarketChartQuery },
}

impl Target for CoinGeckoTarget {
    fn path(&self) -> String {
        match self {
            Self::Global => "/api/v3/global".to_string(),
            Self::SearchTrending => "/api/v3/search/trending".to_string(),
            Self::TopGainersLosers => "/api/v3/coins/top_gainers_losers?vs_currency=usd".to_string(),
            Self::CoinList { query } => build_path_with_query("/api/v3/coins/list", query),
            Self::CoinCategories => "/api/v3/coins/categories/list".to_string(),
            Self::NewCoins => "/api/v3/coins/list/new".to_string(),
            Self::CoinMarkets { query } => build_path_with_query("/api/v3/coins/markets", query),
            Self::Coin { id, query } => build_path_with_query(&format!("/api/v3/coins/{id}"), query),
            Self::CoinByContract { platform_id, contract_address } => format!("/api/v3/coins/{platform_id}/contract/{contract_address}"),
            Self::ExchangeRates => "/api/v3/exchange_rates".to_string(),
            Self::MarketChart { id, query } => build_path_with_query(&format!("/api/v3/coins/{id}/market_chart"), query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            CoinGeckoTarget::CoinByContract {
                platform_id: "ethereum".into(),
                contract_address: "0x1".into()
            }
            .path(),
            "/api/v3/coins/ethereum/contract/0x1"
        );
        assert_eq!(
            CoinGeckoTarget::MarketChart {
                id: "bitcoin".into(),
                query: MarketChartQuery::usd("7", "daily")
            }
            .path(),
            "/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days=7&interval=daily&precision=full"
        );
        assert_eq!(
            CoinGeckoTarget::Coin {
                id: "bitcoin".into(),
                query: CoinQuery::metadata()
            }
            .path(),
            "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true"
        );
    }
}
