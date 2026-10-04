use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{ThrottleCacher, ThrottledTask};
use chain_providers::ChainProviders;
use gem_tracing::info_with_fields;
use storage::{AssetsRepository, Database};
use streamer::{FetchAssetsPayload, StreamProducerQueue, consumer::MessageConsumer};

use crate::assets::AssetClassificationRules;

pub struct FetchAssetsConsumer {
    pub database: Database,
    pub providers: ChainProviders,
    pub throttle: Arc<dyn ThrottleCacher>,
    pub classification_rules: AssetClassificationRules,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn should_consume(&self, payload: &FetchAssetsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle.try_start(ThrottledTask::FetchAssets { asset_id: &payload.asset_id.to_string() }).await
    }

    async fn consume(&self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if payload.asset_id.is_native() {
            return Ok(0);
        }
        let token_id = payload.asset_id.get_token_id()?.clone();
        let asset = self.providers.get_token_data(payload.asset_id.chain, token_id.clone()).await?;
        let classified = self.classification_rules.classified(asset.as_basic_primitive());
        let added = self.database.run(move |client| client.add_assets(vec![classified])).await?;
        if added > 0 {
            self.stream_producer.publish_fetch_asset_status(payload.asset_id.clone()).await?;
        }
        let name = format!("{:?}", asset.name);
        info_with_fields!("fetch asset", chain = payload.asset_id.chain.as_ref(), symbol = asset.symbol.as_str(), name = name.as_str(), added = added);
        Ok(added)
    }
}
