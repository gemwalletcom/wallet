use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use lists::ListProvider;
use primitives::{AssetList, ListId, ListProviderName};

use crate::assets::repository::Repository;

pub struct ListsClient {
    repository: Arc<dyn Repository>,
    providers: HashMap<ListProviderName, Arc<dyn ListProvider>>,
}

impl ListsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, providers: Vec<Arc<dyn ListProvider>>) -> Self {
        Self {
            repository,
            providers: providers.into_iter().map(|provider| (provider.provider(), provider)).collect(),
        }
    }

    pub async fn add_list(&self, id: String, list_id: ListId) -> Result<Option<AssetList>, Box<dyn Error + Send + Sync>> {
        let tag = self.repository.list_tag(id.clone()).await?;
        if tag.as_ref().is_some_and(|tag| tag.list_id.as_ref() != Some(&list_id)) {
            return Ok(None);
        }

        let Some(provider) = self.providers.get(&list_id.provider) else {
            return Ok(None);
        };
        let Some(list) = provider.get_list(&list_id.provider_list_id).await? else {
            return Ok(None);
        };
        let count = self.repository.set_list_assets(id.clone(), list.name.clone(), list_id, list.asset_ids, tag.is_none()).await?;
        let Some(count) = count else {
            return Ok(None);
        };

        Ok(Some(AssetList {
            id,
            name: list.name,
            count: count.try_into()?,
        }))
    }

    pub async fn update_lists(&self, provider: ListProviderName) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let tags = self.repository.list_tags().await?;
        let mut count = 0;
        for tag in tags {
            let Some(list_id) = tag.list_id else {
                continue;
            };
            if list_id.provider == provider && self.add_list(tag.id, list_id).await?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use lists::ListProviderData;
    use primitives::{AssetId, Chain, TagVisibility};
    use storage::Tag;

    use super::*;
    use crate::testkit::{ListAssets, MemoryAssetRepository};

    struct StaticListProvider;

    #[async_trait]
    impl ListProvider for StaticListProvider {
        fn provider(&self) -> ListProviderName {
            ListProviderName::Coingecko
        }

        async fn get_list(&self, _provider_list_id: &str) -> Result<Option<ListProviderData>, Box<dyn Error + Send + Sync>> {
            Ok(Some(ListProviderData {
                name: "Stablecoins".to_string(),
                asset_ids: vec![AssetId::from_chain(Chain::Ethereum)],
            }))
        }
    }

    fn list_id(provider_list_id: &str) -> ListId {
        ListId {
            provider: ListProviderName::Coingecko,
            provider_list_id: provider_list_id.to_string(),
        }
    }

    #[tokio::test]
    async fn test_add_list_creates_new_tag() {
        let repository = Arc::new(MemoryAssetRepository::new(vec![]));
        let client = ListsClient::new(repository.clone(), vec![Arc::new(StaticListProvider)]);

        let list = client.add_list("stablecoins".to_string(), list_id("stablecoins")).await.unwrap().unwrap();

        assert_eq!(list.count, 1);
        assert_eq!(
            repository.list_assets(),
            vec![ListAssets {
                tag_id: "stablecoins".to_string(),
                asset_ids: vec![AssetId::from_chain(Chain::Ethereum)],
                is_new_tag: true,
            }]
        );
    }

    #[tokio::test]
    async fn test_add_list_skips_tag_bound_to_another_list() {
        let tag = Tag {
            id: "stablecoins".to_string(),
            name: "Stablecoins".to_string(),
            visibility: TagVisibility::Public,
            list_id: Some(list_id("other")),
        };
        let repository = Arc::new(MemoryAssetRepository::new(vec![]).with_tags(vec![tag]));
        let client = ListsClient::new(repository.clone(), vec![Arc::new(StaticListProvider)]);

        assert!(client.add_list("stablecoins".to_string(), list_id("stablecoins")).await.unwrap().is_none());
        assert!(repository.list_assets().is_empty());
    }
}
