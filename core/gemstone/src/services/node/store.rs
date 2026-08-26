use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::Chain;
use primitives::node::Node;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNodeStore: Send + Sync {
    async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError>;
    async fn add_node(&self, chain: Chain, node: Node) -> Result<(), GemServiceError>;
    async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError>;
    async fn get_selected_url(&self, chain: Chain) -> Result<Option<String>, GemServiceError>;
    async fn set_selected_url(&self, chain: Chain, url: String) -> Result<(), GemServiceError>;
    async fn clear_selected_url(&self, chain: Chain) -> Result<(), GemServiceError>;
}
