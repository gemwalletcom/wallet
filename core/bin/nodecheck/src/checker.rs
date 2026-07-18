use std::error::Error;

use async_trait::async_trait;

use crate::fixtures::NodeFixture;

#[async_trait]
pub(crate) trait NodeCheck: Send + Sync {
    async fn check_load_balancer(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn check_indexer(&self, fixture: NodeFixture, archival: bool) -> Result<(), Box<dyn Error + Send + Sync>>;
}
