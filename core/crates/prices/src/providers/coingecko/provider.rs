use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use coingecko::{CoinGeckoErrorResponse, CoinInfo, MAX_MARKETS_PER_PAGE, client::CoinGeckoClient, get_coingecko_market_id_for_chain, get_coingecko_platform_id_for_chain};
use gem_client::{Client, ReqwestClient};
use gem_tracing::warn_with_fields;
use primitives::{AssetId, Chain, ChartValue, DurationExt};

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset, PriceProviderAssetMetadata};

use super::mapper::{map_coin_info_metadata, map_coin_mappings, map_coin_markets, map_coins_to_assets, map_coins_to_mappings, map_market_chart};

pub struct CoinGeckoPricesProvider<C: Client = ReqwestClient> {
    client: CoinGeckoClient<C>,
}

impl CoinGeckoPricesProvider<ReqwestClient> {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: CoinGeckoClient::new(api_key),
        }
    }
}

#[async_trait]
impl<C: Client + 'static> PriceAssetsProvider for CoinGeckoPricesProvider<C> {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Coingecko
    }

    async fn get_assets(&self, limit: usize) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let mut markets_by_id: HashMap<String, _> = self
            .client
            .get_all_coin_markets(None, MAX_MARKETS_PER_PAGE, limit.div_ceil(MAX_MARKETS_PER_PAGE))
            .await?
            .into_iter()
            .take(limit)
            .map(|m| (m.id.clone(), m))
            .collect();

        let native_ids: Vec<String> = Chain::all()
            .into_iter()
            .map(get_coingecko_market_id_for_chain)
            .map(str::to_string)
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|id| !markets_by_id.contains_key(id))
            .collect();
        if !native_ids.is_empty() {
            let native_markets = self.client.get_coin_markets_ids(native_ids, MAX_MARKETS_PER_PAGE).await?;
            markets_by_id.extend(native_markets.into_iter().map(|market| (market.id.clone(), market)));
        }

        let coins = self.client.get_coin_list().await?.into_iter().filter(|c| markets_by_id.contains_key(&c.id)).collect();
        Ok(map_coins_to_assets(coins, markets_by_id))
    }

    async fn get_assets_new(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let ids: HashSet<String> = self
            .client
            .get_search_trending()
            .await?
            .get_coins_ids()
            .into_iter()
            .chain(self.client.get_coin_list_new().await?.ids())
            .collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let coins = self.client.get_coin_list().await?.into_iter().filter(|c| ids.contains(&c.id)).collect();
        Ok(map_coins_to_mappings(coins).into_iter().map(|m| PriceProviderAsset::new(m, None)).collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        let (Some(platform_id), Some(token_id)) = (get_coingecko_platform_id_for_chain(asset_id.chain), asset_id.token_id.as_deref()) else {
            return Ok(vec![]);
        };
        let Some(coin_info) = optional_coin(self.client.get_coin_by_contract(platform_id, token_id).await)? else {
            return Ok(vec![]);
        };
        Ok(vec![AssetPriceMapping::new(asset_id.clone(), coin_info.id)])
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        let Some(coin_info) = optional_coin(self.client.get_coin(provider_price_id).await)? else {
            return Ok(vec![]);
        };
        Ok(map_coin_mappings(&coin_info.id, &coin_info.platforms))
    }

    async fn get_assets_metadata(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<PriceProviderAssetMetadata>, Box<dyn Error + Send + Sync>> {
        let grouped = mappings.into_iter().fold(HashMap::new(), |mut grouped, mapping| {
            grouped.entry(mapping.provider_price_id.clone()).or_insert_with(Vec::new).push(mapping);
            grouped
        });
        let mut metadata = Vec::new();
        for (provider_price_id, mappings) in grouped {
            match optional_coin(self.client.get_coin(&provider_price_id).await)? {
                Some(coin_info) => metadata.extend(map_coin_info_metadata(mappings, coin_info)),
                None => {
                    warn_with_fields!(
                        "skip unavailable price asset metadata",
                        provider = PriceProvider::Coingecko.id(),
                        provider_price_id = provider_price_id.as_str()
                    );
                }
            }
        }
        Ok(metadata)
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }

        let by_id = mappings.into_iter().fold(HashMap::<String, Vec<AssetPriceMapping>>::new(), |mut by_id, mapping| {
            by_id.entry(mapping.provider_price_id.clone()).or_default().push(mapping);
            by_id
        });
        let ids: Vec<String> = by_id.keys().cloned().collect();
        let mut out = Vec::with_capacity(by_id.len());
        for chunk in ids.chunks(MAX_MARKETS_PER_PAGE) {
            let coin_markets = self.client.get_coin_markets_ids(chunk.to_vec(), MAX_MARKETS_PER_PAGE).await?;
            out.extend(map_coin_markets(coin_markets, &by_id));
        }
        Ok(out)
    }

    async fn get_charts_daily(&self, provider_price_id: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let chart = self.client.get_market_chart(provider_price_id, "daily", "max").await?;
        Ok(map_market_chart(chart))
    }

    async fn get_charts_hourly(&self, provider_price_id: &str, duration: Duration) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let days = duration.as_days_ceil().max(1).to_string();
        let chart = self.client.get_market_chart(provider_price_id, "hourly", &days).await?;
        Ok(map_market_chart(chart))
    }
}

fn optional_coin(result: Result<CoinInfo, Box<dyn Error + Send + Sync>>) -> Result<Option<CoinInfo>, Box<dyn Error + Send + Sync>> {
    match result {
        Ok(coin) => Ok(Some(coin)),
        Err(error) if error.downcast_ref::<CoinGeckoErrorResponse>().is_some_and(CoinGeckoErrorResponse::is_coin_not_found) => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gem_client::{ClientError, testkit::MockClient};
    use primitives::Chain;

    use super::*;

    const COIN_INFO: &str = r#"{"id":"bitcoin","symbol":"btc","name":"Bitcoin","asset_platform_id":null,"preview_listing":false,"market_cap_rank":1,"market_cap_rank_with_rehypothecated":null,"watchlist_portfolio_users":1000,"platforms":{},"detail_platforms":{},"links":{"homepage":[],"blockchain_site":[],"chat_url":[],"subreddit_url":null,"twitter_screen_name":null,"facebook_username":null,"telegram_channel_identifier":null,"repos_url":{}},"community_data":null,"image":{"thumb":"","small":"","large":""}}"#;

    #[tokio::test]
    async fn test_get_assets_metadata_skips_unavailable_coin() {
        let paths = Arc::new(Mutex::new(Vec::new()));
        let recorded_paths = paths.clone();
        let client = MockClient::new().with_get(move |path| {
            recorded_paths.lock().unwrap().push(path.to_string());
            match path {
                value if value.starts_with("/api/v3/coins/bitcoin?") => Ok(COIN_INFO.as_bytes().to_vec()),
                value if value.starts_with("/api/v3/coins/removed?") => Ok(br#"{"error":"coin not found"}"#.to_vec()),
                _ => Err(ClientError::Http { status: 404, body: vec![] }),
            }
        });
        let provider = CoinGeckoPricesProvider {
            client: CoinGeckoClient::new_with_client(client),
        };
        let mappings = vec![
            AssetPriceMapping::new(Chain::Bitcoin.as_asset_id(), "bitcoin".to_string()),
            AssetPriceMapping::new(Chain::Ethereum.as_asset_id(), "removed".to_string()),
        ];

        let metadata = provider.get_assets_metadata(mappings).await.unwrap();
        let mut paths = paths.lock().unwrap().clone();
        paths.sort();

        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0].asset_id, Chain::Bitcoin.as_asset_id());
        assert_eq!(
            paths,
            vec![
                "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true",
                "/api/v3/coins/removed?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true",
            ]
        );
    }

    #[tokio::test]
    async fn test_get_assets_metadata_returns_transient_error() {
        let client = MockClient::new().with_get(|_| Err(ClientError::Serialization("temporary response error".to_string())));
        let provider = CoinGeckoPricesProvider {
            client: CoinGeckoClient::new_with_client(client),
        };
        let mappings = vec![AssetPriceMapping::new(Chain::Bitcoin.as_asset_id(), "bitcoin".to_string())];

        let error = provider.get_assets_metadata(mappings).await.unwrap_err();

        assert_eq!(error.to_string(), "temporary response error");
    }

    #[tokio::test]
    async fn test_get_mappings_for_asset_id_returns_empty_when_coin_is_unavailable() {
        let client = MockClient::new().with_get(|_| Ok(br#"{"error":"coin not found"}"#.to_vec()));
        let provider = CoinGeckoPricesProvider {
            client: CoinGeckoClient::new_with_client(client),
        };
        let asset_id = AssetId::from_token(Chain::Ethereum, "0x123");

        let mappings = provider.get_mappings_for_asset_id(&asset_id).await.unwrap();

        assert!(mappings.is_empty());
    }
}
