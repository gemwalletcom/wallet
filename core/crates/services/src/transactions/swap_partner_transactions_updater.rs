use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use primitives::SwapProvider;
use primitives::swap::SwapPartnerTransaction;
use storage::{AssetsRepository, Database, SwapPartnerTransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue};
use swapper::partner::{SwapPartnerCursor, SwapPartnerProvider};

#[derive(Clone)]
pub struct SwapPartnerTransactionsUpdater {
    provider: Arc<dyn SwapPartnerProvider>,
    database: Database,
    cacher: CacherClient,
    stream_producer: StreamProducer,
}

impl SwapPartnerTransactionsUpdater {
    pub fn new(provider: Arc<dyn SwapPartnerProvider>, database: Database, cacher: CacherClient, stream_producer: StreamProducer) -> Self {
        Self { provider, database, cacher, stream_producer }
    }

    pub fn provider(&self) -> SwapProvider {
        self.provider.provider()
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider();
        let mut cursor = self.cacher.get_cached_optional::<String>(CacheKey::SwapPartnerCursor(provider.as_ref())).await?;
        let mut count = 0;
        loop {
            let page = self.provider.get_transactions(cursor).await?;
            count += self.store_transactions(page.transactions).await?;
            self.cacher.set_cached(CacheKey::SwapPartnerCursor(provider.as_ref()), &page.cursor.value()).await?;
            match page.cursor {
                SwapPartnerCursor::Next(next) => cursor = Some(next),
                SwapPartnerCursor::Latest(_) => return Ok(count),
            }
        }
    }

    async fn store_transactions(&self, transactions: Vec<SwapPartnerTransaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_ids = transactions.iter().flat_map(SwapPartnerTransaction::asset_ids).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        let existing_ids = self
            .database
            .run({
                let asset_ids = asset_ids.clone();
                move |client| client.get_assets_basic(asset_ids)
            })
            .await?
            .into_iter()
            .map(|asset| asset.asset.id)
            .collect::<HashSet<_>>();
        let missing_ids = asset_ids.into_iter().filter(|asset_id| !existing_ids.contains(asset_id)).collect::<Vec<_>>();
        self.stream_producer.publish_fetch_assets(missing_ids).await?;

        let transactions = transactions
            .into_iter()
            .filter(|transaction| transaction.asset_ids().iter().all(|asset_id| existing_ids.contains(asset_id)))
            .collect::<Vec<_>>();
        Ok(self.database.run(move |client| client.add_swap_partner_transactions(transactions)).await?)
    }
}
