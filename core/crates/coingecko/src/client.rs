use crate::model::{
    Coin, CoinCategory, CoinGeckoResponse, CoinIds, CoinInfo, CoinMarket, CoinMarketsQuery, CoinQuery, CointListQuery, Data, ExchangeRates, Global, MarketChart, SearchTrending,
    TopGainersLosers,
};
use gem_client::{Client, ClientExt, RemoteProviderConfig, ReqwestClient, build_path_with_query, retry};
use primitives::{FiatRate, currency::Currency};
use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;

pub const MAX_MARKETS_PER_PAGE: usize = 250;
const COINGECKO_API_HEADER_KEY: &str = "x-cg-pro-api-key";
const USER_AGENT_VALUE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

#[derive(Debug, Clone)]
pub struct CoinGeckoClient<C: Client> {
    client: C,
}

impl CoinGeckoClient<ReqwestClient> {
    pub fn new(config: RemoteProviderConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        if !config.key.is_empty() {
            headers.insert(COINGECKO_API_HEADER_KEY, HeaderValue::from_str(&config.key).unwrap());
        }
        let reqwest_client = gem_client::builder().default_headers(headers).build().unwrap();

        let client = ReqwestClient::new(config.url, reqwest_client);
        Self { client }
    }
}

impl<C: Client> CoinGeckoClient<C> {
    pub fn new_with_client(client: C) -> Self {
        Self { client }
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        retry(
            || async {
                let response: CoinGeckoResponse<T> = self.client.get(path).await.map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
                match response {
                    CoinGeckoResponse::Success(data) => Ok(data),
                    CoinGeckoResponse::Error(error) => Err(error.into()),
                }
            },
            3,
        )
        .await
    }

    pub async fn get_global(&self) -> Result<Global, Box<dyn Error + Send + Sync>> {
        Ok(self.get_json::<Data<Global>>("/api/v3/global").await?.data)
    }

    pub async fn get_search_trending(&self) -> Result<SearchTrending, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/search/trending";
        self.get_json(path).await
    }

    pub async fn get_top_gainers_losers(&self) -> Result<TopGainersLosers, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/coins/top_gainers_losers?vs_currency=usd";
        self.get_json(path).await
    }

    pub async fn get_coin_list(&self) -> Result<Vec<Coin>, Box<dyn Error + Send + Sync>> {
        let query = CointListQuery { include_platform: true };
        let path = build_path_with_query("/api/v3/coins/list", &query);
        self.get_json(&path).await
    }

    pub async fn get_coin_categories_list(&self) -> Result<Vec<CoinCategory>, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/coins/categories/list";
        self.get_json(path).await
    }

    pub async fn get_coin_list_new(&self) -> Result<CoinIds, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/coins/list/new";
        self.get_json(path).await
    }

    pub async fn get_coin_markets(&self, page: usize, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        self.get_coin_markets_query(Some(page), per_page, None, None).await
    }

    pub async fn get_coin_markets_ids(&self, ids: Vec<String>, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        self.get_coin_markets_query(None, per_page, Some(ids.join(",")), None).await
    }

    async fn get_coin_markets_query(
        &self,
        page: Option<usize>,
        per_page: usize,
        ids: Option<String>,
        category: Option<String>,
    ) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let path = build_path_with_query(
            "/api/v3/coins/markets",
            &CoinMarketsQuery {
                vs_currency: "usd",
                order: "market_cap_desc",
                per_page,
                page,
                sparkline: false,
                locale: "en",
                ids,
                category,
                include_rehypothecated: true,
            },
        );
        self.get_json(&path).await
    }

    pub async fn get_coin(&self, id: &str) -> Result<CoinInfo, Box<dyn Error + Send + Sync>> {
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let base_path = format!("/api/v3/coins/{}", id);
        let path = build_path_with_query(&base_path, &query);
        self.get_json(&path).await
    }

    pub async fn get_coin_by_contract(&self, platform_id: &str, contract_address: &str) -> Result<CoinInfo, Box<dyn Error + Send + Sync>> {
        let path = format!("/api/v3/coins/{platform_id}/contract/{contract_address}");
        self.get_json(&path).await
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        let path = "/api/v3/exchange_rates";
        let rates: ExchangeRates = self.get_json(path).await?;
        let usd_symbol = Currency::USD.as_ref().to_lowercase();
        let usd_rate = rates.rates.get(&usd_symbol).ok_or("Default fiat currency rate not found")?.value;

        let fiat_rates: Vec<FiatRate> = rates
            .rates
            .into_iter()
            .filter(|(_, rate)| rate.rate_type == "fiat")
            .filter_map(|(identifier, rate)| {
                Some(FiatRate {
                    symbol: identifier.to_uppercase().parse().ok()?,
                    rate: rate.value / usd_rate,
                })
            })
            .collect();
        Ok(fiat_rates)
    }

    pub async fn get_all_coin_markets(&self, start_page: Option<usize>, per_page: usize, pages: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let mut all_coin_markets = Vec::new();
        let mut page = start_page.unwrap_or(1);

        loop {
            let coin_markets = self.get_coin_markets(page, per_page).await?;
            let is_empty = coin_markets.is_empty();

            all_coin_markets.extend(coin_markets);

            if is_empty || page == pages {
                break;
            }

            page += 1;
        }

        Ok(all_coin_markets)
    }

    pub async fn get_all_coin_markets_by_category(&self, category: &str, per_page: usize) -> Result<Vec<CoinMarket>, Box<dyn Error + Send + Sync>> {
        let mut all_coin_markets = Vec::new();
        let mut page = 1;

        loop {
            let coin_markets = self.get_coin_markets_query(Some(page), per_page, None, Some(category.to_string())).await?;
            if coin_markets.is_empty() {
                break;
            }

            all_coin_markets.extend(coin_markets);
            page += 1;
        }

        Ok(all_coin_markets)
    }

    pub async fn get_market_chart(&self, coin_id: &str, interval: &str, days: &str) -> Result<MarketChart, Box<dyn Error + Send + Sync>> {
        let path = format!("/api/v3/coins/{}/market_chart?vs_currency=usd&days={}&interval={}&precision=full", coin_id, days, interval);
        self.get_json(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_get_all_coin_markets_by_category_uses_category_query() {
        let client = MockClient::new().with_get(|path| {
            let body = match path {
                "/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=250&page=1&sparkline=false&locale=en&category=xstocks-ecosystem&include_rehypothecated=true" => {
                    r#"[{"id":"tesla-xstock","symbol":"tslax","name":"Tesla xStock","image":"https://example.com/tesla.png"}]"#
                }
                "/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=250&page=2&sparkline=false&locale=en&category=xstocks-ecosystem&include_rehypothecated=true" => {
                    "[]"
                }
                _ => {
                    return Err(gem_client::ClientError::Http {
                        status: 404,
                        body: path.as_bytes().to_vec(),
                    });
                }
            };
            Ok(body.as_bytes().to_vec())
        });
        let client = CoinGeckoClient::new_with_client(client);

        let markets = client.get_all_coin_markets_by_category("xstocks-ecosystem", MAX_MARKETS_PER_PAGE).await.unwrap();

        assert_eq!(markets.len(), 1);
        assert_eq!(markets[0].id, "tesla-xstock");
    }

    #[tokio::test]
    async fn test_get_coin_categories_list_parses_names() {
        let client = MockClient::new().with_get(|path| {
            let body = match path {
                "/api/v3/coins/categories/list" => r#"[{"category_id":"xstocks-ecosystem","name":"xStocks Ecosystem"}]"#,
                _ => {
                    return Err(gem_client::ClientError::Http {
                        status: 404,
                        body: path.as_bytes().to_vec(),
                    });
                }
            };
            Ok(body.as_bytes().to_vec())
        });
        let client = CoinGeckoClient::new_with_client(client);

        let categories = client.get_coin_categories_list().await.unwrap();

        assert_eq!(categories[0].category_id, "xstocks-ecosystem");
        assert_eq!(categories[0].name, "xStocks Ecosystem");
    }

    #[tokio::test]
    async fn test_get_coin_markets_ids_skips_empty_ids() {
        let client = MockClient::new().with_get(|path| {
            Err(gem_client::ClientError::Http {
                status: 500,
                body: path.as_bytes().to_vec(),
            })
        });
        let client = CoinGeckoClient::new_with_client(client);

        let markets = client.get_coin_markets_ids(vec![], MAX_MARKETS_PER_PAGE).await.unwrap();

        assert_eq!(markets.len(), 0);
    }
}
