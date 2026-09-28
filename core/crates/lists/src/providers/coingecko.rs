use std::collections::{HashMap, HashSet};
use std::error::Error;

use async_trait::async_trait;
use coingecko::{CoinGeckoClient, MAX_MARKETS_PER_PAGE, get_asset_ids_for_coin};
use primitives::{AssetId, ListProviderName};

use crate::provider::{ListProvider, ListProviderData};

pub struct CoinGeckoListProvider {
    client: CoinGeckoClient,
}

impl CoinGeckoListProvider {
    pub fn new(client: CoinGeckoClient) -> Self {
        Self { client }
    }

    async fn get_asset_ids_for_coin_ids(&self, coin_ids: &[String]) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        if coin_ids.is_empty() {
            return Ok(vec![]);
        }

        let coin_id_set = coin_ids.iter().cloned().collect::<HashSet<_>>();
        let asset_ids_by_coin_id = self
            .client
            .get_coin_list()
            .await?
            .into_iter()
            .filter(|coin| coin_id_set.contains(&coin.id))
            .map(|coin| {
                let asset_ids = get_asset_ids_for_coin(&coin.id, &coin.platforms);
                (coin.id, asset_ids)
            })
            .collect::<HashMap<_, _>>();

        Ok(coin_ids.iter().filter_map(|coin_id| asset_ids_by_coin_id.get(coin_id)).flatten().cloned().collect())
    }
}

#[async_trait]
impl ListProvider for CoinGeckoListProvider {
    fn provider(&self) -> ListProviderName {
        ListProviderName::Coingecko
    }

    async fn get_list(&self, provider_list_id: &str) -> Result<Option<ListProviderData>, Box<dyn Error + Send + Sync>> {
        let Some(name) = self
            .client
            .get_coin_categories_list()
            .await?
            .into_iter()
            .find(|category| category.category_id == provider_list_id)
            .map(|category| category.name)
        else {
            return Ok(None);
        };

        let coin_ids = self
            .client
            .get_all_coin_markets_by_category(provider_list_id, MAX_MARKETS_PER_PAGE)
            .await?
            .into_iter()
            .map(|market| market.id)
            .collect::<Vec<_>>();

        Ok(Some(ListProviderData {
            name,
            asset_ids: self.get_asset_ids_for_coin_ids(&coin_ids).await?,
        }))
    }
}
