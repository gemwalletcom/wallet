use std::sync::Mutex;

use primitives::Chain;
use primitives::node::Node;

use super::GemNodeStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryNodeStore {
    pub nodes: Mutex<Vec<Node>>,
}

#[async_trait::async_trait]
impl GemNodeStore for MemoryNodeStore {
    async fn get_nodes(&self, _chain: Chain) -> Result<Vec<Node>, GemServiceError> {
        Ok(self.nodes.lock().unwrap().clone())
    }
    async fn add_node(&self, _chain: Chain, node: Node) -> Result<(), GemServiceError> {
        self.nodes.lock().unwrap().push(node);
        Ok(())
    }
    async fn delete_node(&self, _chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.lock().unwrap().retain(|node| node.url != url);
        Ok(())
    }
}
