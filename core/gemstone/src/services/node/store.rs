use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::Chain;
use primitives::node::Node;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemNodeStore: Send + Sync {
    async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError>;
    async fn add_node(&self, chain: Chain, node: Node) -> Result<(), GemServiceError>;
    async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError>;
}
