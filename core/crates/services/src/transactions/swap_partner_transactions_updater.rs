use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use chain_primitives::checksum_address;
use gem_client::{DEFAULT_MAX_RETRIES, default_should_retry, retry};
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
    page_delay: Duration,
}

impl SwapPartnerTransactionsUpdater {
    pub fn new(provider: Arc<dyn SwapPartnerProvider>, database: Database, cacher: CacherClient, stream_producer: StreamProducer, page_delay: Duration) -> Self {
        Self {
            provider,
            database,
            cacher,
            stream_producer,
            page_delay,
        }
    }

    pub fn name(&self) -> &'static str {
        self.provider.name()
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let name = self.name();
        let mut cursor = self.cacher.get_cached_optional::<String>(CacheKey::SwapPartnerCursor(name)).await?;
        let mut count = 0;
        loop {
            let page = retry(|| self.provider.get_transactions(cursor.clone()), DEFAULT_MAX_RETRIES, default_should_retry).await?;
            count += self.store_transactions(page.transactions).await?;
            self.cacher.set_cached(CacheKey::SwapPartnerCursor(name), &page.cursor.value()).await?;
            match page.cursor {
                SwapPartnerCursor::Next(next) => {
                    tokio::time::sleep(self.page_delay).await;
                    cursor = Some(next);
                }
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
            .map(normalize_addresses)
            .collect::<Vec<_>>();
        Ok(self.database.run(move |client| client.add_swap_partner_transactions(transactions)).await?)
    }
}

fn normalize_addresses(transaction: SwapPartnerTransaction) -> SwapPartnerTransaction {
    SwapPartnerTransaction {
        from_address: checksum_address(&transaction.from_address, transaction.from_asset_id.chain),
        to_address: checksum_address(&transaction.to_address, transaction.to_asset_id.chain),
        ..transaction
    }
}
