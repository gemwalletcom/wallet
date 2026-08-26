use async_trait::async_trait;
use primitives::Chain;
use primitives::node::Node;

use super::error::GemNodeError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNodeStore: Send + Sync {
    async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemNodeError>;
    async fn add_node(&self, chain: Chain, node: Node) -> Result<(), GemNodeError>;
    async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemNodeError>;
    async fn get_selected_url(&self, chain: Chain) -> Result<Option<String>, GemNodeError>;
    async fn set_selected_url(&self, chain: Chain, url: String) -> Result<(), GemNodeError>;
    async fn clear_selected_url(&self, chain: Chain) -> Result<(), GemNodeError>;
}
