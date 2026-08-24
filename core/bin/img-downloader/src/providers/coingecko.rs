use super::{config::CoingeckoProviderConfig, mapper::is_native_token, model::AssetImage};
use coingecko::{Coin, CoinGeckoClient, CoinMarket, MAX_MARKETS_PER_PAGE, get_chain_for_coingecko_platform_id, model::SearchTrending};
use gem_client::RemoteProviderConfig;
use std::{collections::HashMap, error::Error};

pub struct CoingeckoProvider {
    client: CoinGeckoClient,
    config: CoingeckoProviderConfig,
}

impl CoingeckoProvider {
    pub fn new(provider: RemoteProviderConfig, config: CoingeckoProviderConfig) -> Self {
        Self {
            client: CoinGeckoClient::new(provider),
            config,
        }
    }

    pub async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        if self.config.top_count == 0 {
            return Ok(vec![]);
        }

        let pages = self.config.top_count.div_ceil(MAX_MARKETS_PER_PAGE);
        let markets = self
            .client
            .get_all_coin_markets(None, MAX_MARKETS_PER_PAGE, pages)
            .await?
            .into_iter()
            .take(self.config.top_count)
            .collect();
        let coins = self.client.get_coin_list().await?;
        Ok(Self::map_market_images(markets, Self::coins_by_id(coins)))
    }

    pub async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let trending = self.client.get_search_trending().await?;
        let coins = self.client.get_coin_list().await?;
        Ok(Self::map_trending_images(trending, Self::coins_by_id(coins)))
    }

    pub async fn get_asset_images(&self, coin_id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let coin_info = self.client.get_coin(coin_id).await?;
        Ok(Self::map_platform_images(coin_info.platforms, coin_info.image.large))
    }

    fn map_market_images(markets: Vec<CoinMarket>, mut coins_by_id: HashMap<String, Coin>) -> Vec<AssetImage> {
        markets
            .into_iter()
            .flat_map(|market| {
                coins_by_id
                    .remove(&market.id)
                    .map(|coin| Self::map_platform_images(coin.platforms, market.image))
                    .unwrap_or_default()
            })
            .collect()
    }

    fn map_trending_images(trending: SearchTrending, mut coins_by_id: HashMap<String, Coin>) -> Vec<AssetImage> {
        trending
            .coins
            .into_iter()
            .flat_map(|trending_item| {
                let image_url = trending_item.item.large.unwrap_or_default();
                coins_by_id
                    .remove(&trending_item.item.id)
                    .map(|coin| Self::map_platform_images(coin.platforms, image_url))
                    .unwrap_or_default()
            })
            .collect()
    }

    fn coins_by_id(coins: Vec<Coin>) -> HashMap<String, Coin> {
        coins.into_iter().map(|coin| (coin.id.clone(), coin)).collect()
    }

    fn map_platform_images(platforms: HashMap<String, Option<String>>, image_url: String) -> Vec<AssetImage> {
        if image_url.is_empty() {
            return vec![];
        }

        platforms
            .into_iter()
            .filter(|(platform, _)| !platform.is_empty())
            .filter_map(|(platform, address)| {
                let chain = get_chain_for_coingecko_platform_id(&platform);
                let address = address?;
                let chain = chain?;
                if address.is_empty() {
                    return None;
                }
                let image = AssetImage {
                    chain,
                    token_id: address,
                    image_url: image_url.clone(),
                };
                (!is_native_token(&image)).then_some(image)
            })
            .collect()
    }
}
