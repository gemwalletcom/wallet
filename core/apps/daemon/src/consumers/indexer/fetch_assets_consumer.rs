use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use gem_tracing::info_with_fields;
use settings_chain::ChainProviders;
use storage::{AssetsRepository, Database};
use streamer::{FetchAssetsPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

use crate::asset_spam::AssetClassificationRules;

pub struct FetchAssetsConsumer {
    pub database: Database,
    pub providers: ChainProviders,
    pub cacher: CacherClient,
    pub classification_rules: AssetClassificationRules,
    pub stream_producer: StreamProducer,
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn should_process(&self, payload: &FetchAssetsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchAssets(&payload.asset_id.to_string())).await
    }

    async fn process(&self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if payload.asset_id.is_native() {
            return Ok(0);
        }
        let token_id = payload.asset_id.get_token_id()?.clone();
        let asset = self.providers.get_token_data(payload.asset_id.chain, token_id.to_string()).await?;
        let added = self.database.assets()?.add_assets(vec![self.classification_rules.apply(asset.as_basic_primitive())])?;
        if added > 0 {
            self.stream_producer.publish_fetch_asset_status(payload.asset_id.clone()).await?;
        }
        let name = format!("{:?}", asset.name);
        info_with_fields!(
            "fetch asset",
            chain = payload.asset_id.chain.as_ref(),
            symbol = asset.symbol.as_str(),
            name = name.as_str(),
            added = added
        );
        Ok(added)
    }
}
